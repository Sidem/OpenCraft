/**
 * One co-op room: a Durable Object that relays WebRTC offers, answers and ICE candidates between the
 * host and each joiner over WebSockets. It holds no game state and stores nothing: who is connected
 * lives in each socket's attachment, so the object can hibernate between messages (idle rooms cost
 * nothing).
 *
 * Protocol (JSON text messages):
 *   host → room   { type: 'signal', to: peer, data }     room → host   { type: 'joined' | 'left', peer }
 *   joiner → room { type: 'signal', data }                             { type: 'signal', from: peer, data }
 *   room → joiner { type: 'welcome', peer }, { type: 'signal', from: 'host', data }
 * Close codes: 4000 a host is already here, 4004 no host (no such room), 4001 the host left,
 * 4008 the room is full. Once its data channel is open a joiner closes its socket; the host keeps
 * its own open for later joiners.
 */

import { DurableObject } from 'cloudflare:workers';

/** Joiners connected at once (the game itself refuses players past its own cap). */
const MAX_JOINERS = 8;
/** Offers and candidates are a few KB; anything much bigger is not signalling. */
const MAX_MESSAGE = 16_384;

interface Who {
  host: boolean;
  peer: string;
}

export class Room extends DurableObject {
  async fetch(request: Request): Promise<Response> {
    const role = new URL(request.url).searchParams.get('role');
    if (role !== 'host' && role !== 'join') return new Response('role must be host or join', { status: 400 });
    const [client, server] = Object.values(new WebSocketPair());
    const host = this.ctx.getWebSockets('host')[0];

    if (role === 'host' && host) return this.refuse(client, server, 4000, 'a host is already in this room');
    if (role === 'join' && !host) return this.refuse(client, server, 4004, 'no game is hosted with this code');
    if (role === 'join' && this.ctx.getWebSockets('joiner').length >= MAX_JOINERS) {
      return this.refuse(client, server, 4008, 'this room is full');
    }

    const who: Who = { host: role === 'host', peer: role === 'host' ? 'host' : crypto.randomUUID().slice(0, 8) };
    this.ctx.acceptWebSocket(server, [who.host ? 'host' : 'joiner', `peer:${who.peer}`]);
    server.serializeAttachment(who);
    if (!who.host) {
      server.send(JSON.stringify({ type: 'welcome', peer: who.peer }));
      host.send(JSON.stringify({ type: 'joined', peer: who.peer }));
    }
    return new Response(null, { status: 101, webSocket: client });
  }

  async webSocketMessage(ws: WebSocket, message: string | ArrayBuffer): Promise<void> {
    if (typeof message !== 'string' || message.length > MAX_MESSAGE) return;
    let msg: { type?: unknown; to?: unknown; data?: unknown };
    try {
      msg = JSON.parse(message);
    } catch {
      return;
    }
    if (msg.type !== 'signal') return;
    const who = ws.deserializeAttachment() as Who;
    const target = who.host ? (typeof msg.to === 'string' ? this.peer(msg.to) : undefined) : this.peer('host');
    target?.send(JSON.stringify({ type: 'signal', from: who.peer, data: msg.data }));
  }

  async webSocketClose(ws: WebSocket, code: number, reason: string): Promise<void> {
    const who = ws.deserializeAttachment() as Who;
    closeQuietly(ws, code, reason);
    if (who.host) {
      for (const joiner of this.ctx.getWebSockets('joiner')) closeQuietly(joiner, 4001, 'the host left');
    } else {
      this.peer('host')?.send(JSON.stringify({ type: 'left', peer: who.peer }));
    }
  }

  async webSocketError(ws: WebSocket): Promise<void> {
    await this.webSocketClose(ws, 1011, 'error');
  }

  private peer(id: string): WebSocket | undefined {
    return this.ctx.getWebSockets(`peer:${id}`)[0];
  }

  /** Accepts the socket only to close it with a code and reason the game can show. */
  private refuse(client: WebSocket, server: WebSocket, code: number, reason: string): Response {
    server.accept();
    server.close(code, reason);
    return new Response(null, { status: 101, webSocket: client });
  }
}

/** Codes 1005 and 1006 only describe a close and can't be sent; a socket may already be closed. */
function closeQuietly(ws: WebSocket, code: number, reason: string): void {
  try {
    ws.close(code === 1005 || code === 1006 ? 1000 : code, reason);
  } catch {
    // Already closed.
  }
}
