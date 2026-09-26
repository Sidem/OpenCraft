/**
 * OpenCraft signalling Worker (co-op, DEV_PLAN section 2): introduces co-op peers so they can open WebRTC
 * data channels to each other, then steps aside. No game logic or game data passes through here.
 *
 *   POST /room         → { code }: a fresh room code for a host
 *   GET  /room/<code>  → WebSocket into that room (?role=host or ?role=join); protocol in room.ts
 *   GET  /ice          → { iceServers }: Cloudflare STUN plus short-lived Cloudflare TURN credentials
 *
 * Secrets (`npx wrangler secret put`): TURN_KEY_ID and TURN_KEY_API_TOKEN; without them /ice gives
 * STUN only. Browsers may call it only from ALLOWED_ORIGINS (wrangler.jsonc) or localhost; the origin
 * check stops other websites, not scripts, so the TURN key itself never leaves the Worker.
 * Deploying: docs/WORKFLOW.md section 4.
 */

export { Room } from './room';

export interface Env {
  ROOMS: DurableObjectNamespace;
  ALLOWED_ORIGINS: string;
  TURN_KEY_ID?: string;
  TURN_KEY_API_TOKEN?: string;
}

/** Room codes: six characters from an alphabet without look-alikes (0/O, 1/I/L). */
const CODE_ALPHABET = 'ABCDEFGHJKMNPQRSTUVWXYZ23456789';
const CODE_LENGTH = 6;
const CODE_PATTERN = new RegExp(`^[${CODE_ALPHABET}]{${CODE_LENGTH}}$`);
/** Lifetime of TURN credentials, in seconds: longer than any play session needs to connect. */
const TURN_TTL = 86_400;
const STUN: IceServer[] = [{ urls: 'stun:stun.cloudflare.com:3478' }];
const LOCAL_ORIGIN = /^http:\/\/(localhost|127\.0\.0\.1)(:\d+)?$/;

interface IceServer {
  urls: string | string[];
  username?: string;
  credential?: string;
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const origin = request.headers.get('Origin');
    if (origin && !originAllowed(origin, env)) return new Response('origin not allowed', { status: 403 });
    const url = new URL(request.url);
    const parts = url.pathname.split('/').filter(Boolean);

    if (request.method === 'OPTIONS') return cors(new Response(null, { status: 204 }), origin);
    if (request.method === 'POST' && url.pathname === '/room') return cors(json({ code: newCode() }), origin);
    if (request.method === 'GET' && url.pathname === '/ice') return cors(json({ iceServers: await ice(env) }), origin);
    if (request.method === 'GET' && parts.length === 2 && parts[0] === 'room') {
      const code = parts[1].toUpperCase();
      if (!CODE_PATTERN.test(code)) return new Response('bad room code', { status: 400 });
      if (request.headers.get('Upgrade') !== 'websocket') return new Response('expected a WebSocket', { status: 426 });
      return env.ROOMS.get(env.ROOMS.idFromName(code)).fetch(request);
    }
    return new Response('not found', { status: 404 });
  },
} satisfies ExportedHandler<Env>;

function originAllowed(origin: string, env: Env): boolean {
  return LOCAL_ORIGIN.test(origin) || env.ALLOWED_ORIGINS.split(',').some((o) => o.trim() === origin);
}

function newCode(): string {
  const bytes = crypto.getRandomValues(new Uint8Array(CODE_LENGTH));
  return Array.from(bytes, (b) => CODE_ALPHABET[b % CODE_ALPHABET.length]).join('');
}

/** STUN plus fresh TURN credentials, or STUN alone when TURN isn't set up or doesn't answer. */
async function ice(env: Env): Promise<IceServer[]> {
  if (!env.TURN_KEY_ID || !env.TURN_KEY_API_TOKEN) return STUN;
  const r = await fetch(
    `https://rtc.live.cloudflare.com/v1/turn/keys/${env.TURN_KEY_ID}/credentials/generate-ice-servers`,
    {
      method: 'POST',
      headers: { Authorization: `Bearer ${env.TURN_KEY_API_TOKEN}`, 'Content-Type': 'application/json' },
      body: JSON.stringify({ ttl: TURN_TTL }),
    },
  );
  if (!r.ok) return STUN;
  const { iceServers } = (await r.json()) as { iceServers: IceServer[] };
  // Browsers block port 53, and a TURN URL on it only times out.
  return iceServers.map((s) => ({ ...s, urls: [s.urls].flat().filter((u) => !/:53\b/.test(u)) }));
}

function json(body: unknown): Response {
  return new Response(JSON.stringify(body), { headers: { 'Content-Type': 'application/json' } });
}

function cors(response: Response, origin: string | null): Response {
  if (origin) {
    response.headers.set('Access-Control-Allow-Origin', origin);
    response.headers.set('Access-Control-Allow-Methods', 'GET, POST, OPTIONS');
    response.headers.set('Vary', 'Origin');
  }
  return response;
}
