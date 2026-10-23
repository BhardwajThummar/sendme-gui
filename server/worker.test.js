import test from "node:test";
import assert from "node:assert/strict";
import worker from "./worker.js";

// Minimal stand-ins for the KV namespace and the rate limiter binding.
const makeEnv = ({ limitAfter = Infinity } = {}) => {
  const store = new Map();
  const puts = [];
  let calls = 0;
  return {
    store,
    puts,
    CODES: {
      get: async (k) => store.get(k) ?? null,
      put: async (k, v, opts) => {
        store.set(k, v);
        puts.push({ k, opts });
      },
    },
    LOOKUP_LIMITER: { limit: async () => ({ success: ++calls <= limitAfter }) },
    CREATE_LIMITER: { limit: async () => ({ success: true }) },
  };
};

const call = (env, method, path, body) =>
  worker.fetch(
    new Request(`https://x.test${path}`, {
      method,
      body: body === undefined ? undefined : typeof body === "string" ? body : JSON.stringify(body),
    }),
    env,
  );

const create = async (env, blobs = ["t1", "t2"]) => (await (await call(env, "POST", "/api/code/", { blobs })).json()).code;

test("root answers ok (the app's reachability probe)", async () => {
  const res = await call(makeEnv(), "GET", "/");
  assert.equal(res.status, 200);
  assert.equal(await res.text(), "ok");
});

test("creates an 8-character code from the unambiguous alphabet and expires it", async () => {
  const env = makeEnv();
  const code = await create(env);
  assert.match(code, /^[A-HJ-NP-Z2-9]{8}$/);
  assert.equal(env.puts[0].opts.expirationTtl, 3600);
});

test("looks a code up, in any letter case", async () => {
  const env = makeEnv();
  const code = await create(env, ["a", "b"]);
  for (const c of [code, code.toLowerCase()]) {
    const res = await call(env, "GET", `/api/code/${c}`);
    assert.deepEqual(await res.json(), { blobs: ["a", "b"] });
  }
});

test("unknown code is a 404", async () => {
  assert.equal((await call(makeEnv(), "GET", "/api/code/ZZZZZZZZ")).status, 404);
});

test("rejects bad input", async () => {
  const env = makeEnv();
  const bad = [
    "not json",
    { blobs: [] },
    { blobs: [1] },
    { blobs: [""] },
    { blobs: ["x".repeat(2001)] },
    { blobs: Array(51).fill("x") },
    {},
  ];
  for (const body of bad) {
    assert.equal((await call(env, "POST", "/api/code/", body)).status, 400, JSON.stringify(body)?.slice(0, 40));
  }
  assert.equal(env.store.size, 0);
});

test("lookups are rate limited per IP", async () => {
  const env = makeEnv({ limitAfter: 3 });
  const code = await create(env);
  const statuses = [];
  for (let i = 0; i < 5; i++) statuses.push((await call(env, "GET", `/api/code/${code}`)).status);
  assert.deepEqual(statuses, [200, 200, 200, 429, 429]);
});

test("works without the limiter bindings (local dev)", async () => {
  const env = makeEnv();
  delete env.LOOKUP_LIMITER;
  delete env.CREATE_LIMITER;
  const code = await create(env);
  assert.equal((await call(env, "GET", `/api/code/${code}`)).status, 200);
});

test("never overwrites a live code: a collision picks a fresh one", async () => {
  const env = makeEnv();
  env.store.set("AAAAAAAA", JSON.stringify(["keep"])); // byte 0 maps to "A"
  const real = crypto.getRandomValues.bind(crypto);
  let calls = 0;
  crypto.getRandomValues = (arr) => (arr.fill(calls++ === 0 ? 0 : 1), arr); // 1st draw collides
  try {
    const code = await create(env, ["other"]);
    assert.equal(code, "BBBBBBBB");
    assert.deepEqual(JSON.parse(env.store.get("AAAAAAAA")), ["keep"]);
  } finally {
    crypto.getRandomValues = real;
  }
});

test("unknown routes are 404", async () => {
  assert.equal((await call(makeEnv(), "GET", "/nope")).status, 404);
  assert.equal((await call(makeEnv(), "DELETE", "/api/code/ABCDEFGH")).status, 404);
});
