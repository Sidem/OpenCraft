// A connection to one other peer: ordered, reliable byte messages. Co-op (`host.ts`, `client.ts`) only
// ever talks through this, so the way peers connect can change underneath: `broadcast.ts` joins tabs of
// one browser, `webrtc.ts` joins machines. To add a way: implement `Transport` and hand the host its
// side of each new peer.

export interface Transport {
  send(bytes: Uint8Array): void;
  /** Set by the owner; called for every message from the other end, in order. */
  onMessage: (bytes: Uint8Array) => void;
  /** Called once when the other end closes or goes away (not after this end's own `close`). */
  onClose: () => void;
  close(): void;
}
