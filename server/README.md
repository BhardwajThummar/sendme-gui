# Code server (optional)

Maps a short 6-character code to the sender's ticket(s), so people don't have
to share a long ticket. The app works without it: tickets and QR codes always
work, and the app hides the code field when this server isn't reachable.

Runs on Cloudflare's free tier (Worker + KV), with a free `*.workers.dev`
address, so there is no server or domain to maintain.

## Deploy

```sh
cd server
npx wrangler login     # once
npx wrangler deploy    # prints https://sendme-codes.<you>.workers.dev
```

Then set that address as `BASE_URL` in the app's `.env` (see `.env.example`).

## Try it locally

```sh
cd server && npx wrangler dev
curl -X POST localhost:8787/api/code/ -d '{"blobs":["abc"]}'   # {"code":"K7M2QX"}
curl localhost:8787/api/code/K7M2QX                              # {"blobs":["abc"]}
```

## Limits

Codes expire after an hour. Anyone can create codes (the free KV tier allows
about 1,000 writes a day), so add a Cloudflare rate-limit rule if it is abused.
