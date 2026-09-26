// Co-op between tabs of one browser over a `BroadcastChannel` named after the room: the dev loop,
// with no server or network involved. Every tab on the channel hears every post, so each post names
// its sender and receiver, and each end keeps only what is addressed to it. A `null` payload means
// the sender closed. A tab that closes without saying so is noticed by the sessions' silence timeouts.

import type { Transport } from './transport';

interface Post {
  from: string;
  to: string;
  data: Uint8Array | null;
}

const HOST = 'host';

/** Host: calls `onPeer` with a transport for each tab that joins `room`. Returns a function that
 * stops listening and closes every peer. */
export function listenBroadcast(room: string, onPeer: (t: Transport) => void): () => void {
  const channel = new BroadcastChannel(channelName(room));
  const peers = new Map<string, ChannelEnd>();
  channel.onmessage = (e: MessageEvent<Post>) => {
    const { from, to, data } = e.data;
    if (to !== HOST) return;
    let peer = peers.get(from);
    if (!peer && data) {
      peer = new ChannelEnd(channel, HOST, from, () => peers.delete(from));
      peers.set(from, peer);
      onPeer(peer);
    }
    peer?.receive(data);
  };
  return () => {
    for (const peer of [...peers.values()]) peer.close();
    channel.close();
  };
}

/** Client: a transport to the host of `room`. */
export function connectBroadcast(room: string): Transport {
  const channel = new BroadcastChannel(channelName(room));
  const end = new ChannelEnd(channel, crypto.randomUUID(), HOST, () => channel.close());
  channel.onmessage = (e: MessageEvent<Post>) => {
    if (e.data.to === end.self && e.data.from === HOST) end.receive(e.data.data);
  };
  return end;
}

function channelName(room: string): string {
  return `opencraft-coop-${room}`;
}

class ChannelEnd implements Transport {
  onMessage = (_bytes: Uint8Array) => {};
  onClose = () => {};
  private closed = false;

  constructor(
    private readonly channel: BroadcastChannel,
    readonly self: string,
    private readonly peer: string,
    private readonly done: () => void,
  ) {}

  send(bytes: Uint8Array): void {
    if (!this.closed) this.post(bytes);
  }

  close(): void {
    if (this.closed) return;
    this.post(null);
    this.finish();
  }

  receive(data: Uint8Array | null): void {
    if (this.closed) return;
    if (data) {
      this.onMessage(data);
    } else {
      this.finish();
      this.onClose();
    }
  }

  private post(data: Uint8Array | null): void {
    this.channel.postMessage({ from: this.self, to: this.peer, data } satisfies Post);
  }

  private finish(): void {
    this.closed = true;
    this.done();
  }
}
