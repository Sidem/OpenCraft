// Co-op between browsers on different machines: one WebRTC data channel (reliable, ordered) per client,
// set up through the signalling Worker (`signal.ts`); `session.ts` starts it. The host keeps its room socket open for later
// joiners and makes each an offer; a joiner answers, and once its channel is open closes its socket.
// Candidates that arrive before the offer or answer wait for it. `relay` forces the connection through a
// TURN relay (`iceTransportPolicy: 'relay'`, for testing; needs TURN set up on the Worker).
// Messages are split into `CHUNK` pieces, each led by one byte (1: more follows, 0: last), because
// browsers cap one data channel message at 256 KB and a join snapshot can be larger.

import { closeReason, fetchIce, newRoom, openRoom } from './signal';
import type { Transport } from './transport';

/** Bytes of message per data channel send: well under every browser's limit. */
const CHUNK = 16 * 1024 - 1;
/** How long a joiner waits for its channel to open. */
const CONNECT_TIMEOUT_MS = 15_000;
/** How long a host waits before reopening a room socket that closed. */
const RECONNECT_MS = 2000;
const NO_CONNECTION = "Couldn't connect to the host. One of your networks may block direct connections.";

interface Signal {
  sdp?: RTCSessionDescriptionInit;
  candidate?: RTCIceCandidateInit;
}

/** Host: opens a room and calls `onPeer` with a transport for each joiner whose channel opens.
 * Returns the room code and a function that closes the room (open channels stay). */
export async function hostWebRtc(onPeer: (t: Transport) => void, relay: boolean): Promise<{ code: string; stop: () => void }> {
  const [code, iceServers] = await Promise.all([newRoom(), fetchIce()]);
  const connecting = new Map<string, Negotiation>();
  let socket: WebSocket;
  let stopped = false;
  const connect = () => {
    socket = openRoom(code, 'host');
    const send = (to: string, data: Signal) => socket.send(JSON.stringify({ type: 'signal', to, data }));
    socket.onmessage = (e: MessageEvent<string>) => {
      const m = JSON.parse(e.data) as { type: string; peer?: string; from?: string; data?: Signal };
      if (m.type === 'joined' && m.peer) {
        const peer = m.peer;
        const n = new Negotiation(iceServers, relay, (data) => send(peer, data));
        connecting.set(peer, n);
        const channel = n.pc.createDataChannel('game', { ordered: true });
        channel.onopen = () => {
          connecting.delete(peer);
          onPeer(new ChannelTransport(n.pc, channel));
        };
        void n.offer();
      } else if (m.type === 'signal' && m.from && m.data) {
        void connecting.get(m.from)?.receive(m.data);
      } else if (m.type === 'left' && m.peer) {
        connecting.get(m.peer)?.pc.close();
        connecting.delete(m.peer);
      }
    };
    socket.onclose = () => {
      if (!stopped) setTimeout(connect, RECONNECT_MS);
    };
  };
  connect();
  const stop = () => {
    stopped = true;
    socket.close(1000, 'the host stopped hosting');
    for (const n of connecting.values()) n.pc.close();
  };
  return { code, stop };
}

/** Joiner: a transport to the host of room `code`. Throws a message a player can read. */
export async function joinWebRtc(code: string, relay: boolean): Promise<Transport> {
  const iceServers = await fetchIce();
  return new Promise((resolve, reject) => {
    const socket = openRoom(code, 'join');
    const n = new Negotiation(iceServers, relay, (data) => socket.send(JSON.stringify({ type: 'signal', data })));
    let done = false;
    const fail = (reason: string) => {
      if (done) return;
      done = true;
      clearTimeout(timer);
      socket.close();
      n.pc.close();
      reject(new Error(reason));
    };
    const timer = setTimeout(() => fail(NO_CONNECTION), CONNECT_TIMEOUT_MS);
    socket.onmessage = (e: MessageEvent<string>) => {
      const m = JSON.parse(e.data) as { type: string; data?: Signal };
      if (m.type === 'signal' && m.data) void n.receive(m.data);
    };
    socket.onclose = (e) => fail(closeReason(e.code));
    n.pc.ondatachannel = (e) => {
      const channel = e.channel;
      channel.onopen = () => {
        if (done) return;
        done = true;
        clearTimeout(timer);
        socket.onclose = null;
        socket.close(1000, 'connected');
        resolve(new ChannelTransport(n.pc, channel));
      };
    };
  });
}

/** One peer connection being set up: sends its offer or answer and candidates through `send`. */
class Negotiation {
  readonly pc: RTCPeerConnection;
  private readonly early: RTCIceCandidateInit[] = [];

  constructor(iceServers: RTCIceServer[], relay: boolean, private readonly send: (data: Signal) => void) {
    this.pc = new RTCPeerConnection({ iceServers, iceTransportPolicy: relay ? 'relay' : 'all' });
    this.pc.onicecandidate = (e) => {
      if (e.candidate) this.send({ candidate: e.candidate.toJSON() });
    };
  }

  async offer(): Promise<void> {
    await this.pc.setLocalDescription();
    this.send({ sdp: this.pc.localDescription!.toJSON() });
  }

  async receive(data: Signal): Promise<void> {
    try {
      if (data.sdp) {
        await this.pc.setRemoteDescription(data.sdp);
        for (const c of this.early.splice(0)) await this.pc.addIceCandidate(c);
        if (data.sdp.type === 'offer') {
          await this.pc.setLocalDescription();
          this.send({ sdp: this.pc.localDescription!.toJSON() });
        }
      } else if (data.candidate) {
        if (this.pc.remoteDescription) await this.pc.addIceCandidate(data.candidate);
        else this.early.push(data.candidate);
      }
    } catch (err) {
      console.warn('co-op: a WebRTC signal was refused', err);
    }
  }
}

class ChannelTransport implements Transport {
  onMessage = (_bytes: Uint8Array) => {};
  onClose = () => {};
  private readonly parts: Uint8Array[] = [];
  private closed = false;

  constructor(
    private readonly pc: RTCPeerConnection,
    private readonly channel: RTCDataChannel,
  ) {
    channel.binaryType = 'arraybuffer';
    channel.onmessage = (e: MessageEvent<ArrayBuffer>) => this.receive(new Uint8Array(e.data));
    channel.onclose = () => this.lost();
    pc.onconnectionstatechange = () => {
      if (pc.connectionState === 'failed' || pc.connectionState === 'closed') this.lost();
    };
  }

  send(bytes: Uint8Array): void {
    if (this.closed || this.channel.readyState !== 'open') return;
    let at = 0;
    do {
      const end = Math.min(at + CHUNK, bytes.length);
      const piece = new Uint8Array(end - at + 1);
      piece[0] = end < bytes.length ? 1 : 0;
      piece.set(bytes.subarray(at, end), 1);
      this.channel.send(piece);
      at = end;
    } while (at < bytes.length);
  }

  close(): void {
    this.closed = true;
    this.channel.close();
    this.pc.close();
  }

  private receive(piece: Uint8Array): void {
    this.parts.push(piece.subarray(1));
    if (piece[0] !== 0) return;
    const size = this.parts.reduce((n, p) => n + p.length, 0);
    const whole = new Uint8Array(size);
    let at = 0;
    for (const p of this.parts.splice(0)) {
      whole.set(p, at);
      at += p.length;
    }
    this.onMessage(whole);
  }

  private lost(): void {
    if (this.closed) return;
    this.closed = true;
    this.pc.close();
    this.onClose();
  }
}
