# Security

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use the repository's **Security** tab, then
**Report a vulnerability**, to write to the maintainer privately. If that option is missing, open an issue that says
only that you have a security report (no details) and the maintainer will arrange a private channel.

## Scope

OpenCraft runs entirely in the browser. What could be affected:

- **Imported worlds** (`.ocworld` files) and saves are parsed by the engine. Malformed files should be rejected,
  never crash the page or run code.
- **Co-op** is peer to peer (WebRTC). The Cloudflare Worker in `signal/` only introduces players and relays no game
  data. Reports about the Worker, room codes or a host or guest abusing the lockstep protocol are in scope.
- **Dependencies** are few (`wasm-bindgen`, `rustc-hash`, `console_error_panic_hook`, Vite and TypeScript for the build).

There is no account system, server-side storage or payment handling. Worlds are stored only in the player's browser.
