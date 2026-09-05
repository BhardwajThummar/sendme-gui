# Code-exchange server API

Setting `BASE_URL` points the app at a small backend that maps a
short, human-typeable code to one or more Iroh blob tickets, so senders
don't have to share the (long) raw ticket text.

This is entirely optional. With no `BASE_URL` set, the app shares the raw
ticket(s) directly as text and a QR code instead. Anyone can implement
this contract to self-host a code-exchange server; the client code lives in
[`src-tauri/src/sendme.rs`](../src-tauri/src/sendme.rs) (`create_blobs` /
`get_blob`).

## `POST {BASE_URL}/api/code/`

Request body:

```json
{ "blobs": ["<ticket 1>", "<ticket 2>", "..."] }
```

Response body:

```json
{ "code": "123456" }
```

The server generates a code, stores the ticket list against it, and
returns the code. Codes should be short-lived and single-purpose — there's
no update or delete endpoint in this contract.

## `GET {BASE_URL}/api/code/{code}`

Response body:

```json
{ "blobs": ["<ticket 1>", "<ticket 2>", "..."] }
```

Looks up the tickets previously stored under `code`. Return a 404 (or any
non-2xx status) if the code is unknown or has expired — the client surfaces
this as a download error.

## Notes for implementers

- The client sends/expects plain ticket strings; the server doesn't need to
  understand Iroh's ticket format, just store and return them as opaque
  strings.
- Codes must not collide with the raw-ticket format the client also
  accepts (see `looks_like_raw_tickets` in `sendme.rs`) — a short numeric
  or alphanumeric code (the reference client uses a fixed length from
  `FILE_TRANSFER_CONFIG.CODE_LENGTH` on the frontend) naturally avoids this
  since it won't parse as a `BlobTicket`.
