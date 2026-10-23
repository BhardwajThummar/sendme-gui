// Code-exchange server for sendme-gui (see docs/SERVER_API.md).
// Cloudflare Worker + KV: free tier, no server to run.

// No 0/O/1/I so codes survive being read aloud; 32 divides 256, so no modulo bias.
const ALPHABET = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const CODE_LENGTH = 8; // keep in sync with FILE_TRANSFER_CONFIG.CODE_LENGTH
const TTL_SECONDS = 60 * 60; // codes expire after an hour
const MAX_BLOBS = 50;
const MAX_BLOB_LENGTH = 2000;

const json = (body, status = 200) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { "content-type": "application/json" },
  });

const newCode = () =>
  Array.from(crypto.getRandomValues(new Uint8Array(CODE_LENGTH)), (b) => ALPHABET[b % 32]).join("");

export default {
  async fetch(request, env) {
    const { pathname } = new URL(request.url);

    // Reachability probe used by the app to decide whether to offer short codes.
    if (pathname === "/") return new Response("ok");

    if (request.method === "POST" && pathname === "/api/code/") {
      let blobs;
      try {
        ({ blobs } = await request.json());
      } catch {
        return json({ error: "invalid json" }, 400);
      }
      const valid =
        Array.isArray(blobs) &&
        blobs.length > 0 &&
        blobs.length <= MAX_BLOBS &&
        blobs.every((b) => typeof b === "string" && b.length > 0 && b.length <= MAX_BLOB_LENGTH);
      if (!valid) return json({ error: "invalid blobs" }, 400);

      // 32^8 codes, so a collision is rare; retry rather than overwrite a live code.
      for (let i = 0; i < 5; i++) {
        const code = newCode();
        if (await env.CODES.get(code)) continue;
        await env.CODES.put(code, JSON.stringify(blobs), { expirationTtl: TTL_SECONDS });
        return json({ code });
      }
      return json({ error: "try again" }, 503);
    }

    const match = pathname.match(/^\/api\/code\/([A-Za-z0-9]+)$/);
    if (request.method === "GET" && match) {
      const stored = await env.CODES.get(match[1].toUpperCase());
      return stored ? json({ blobs: JSON.parse(stored) }) : json({ error: "not found" }, 404);
    }

    return json({ error: "not found" }, 404);
  },
};
