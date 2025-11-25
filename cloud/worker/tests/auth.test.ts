import { strict as assert } from "node:assert";
import test from "node:test";
import worker from "../src/worker";
import { FakeR2Bucket } from "./helpers/fakeR2";
import { verifyJwt } from "../src/jwt";

const TEST_SECRET = "test-secret";

function createEnv() {
  const bucket = new FakeR2Bucket();
  return { CROSSSAVE_R2: bucket as unknown as R2Bucket, JWT_SECRET: TEST_SECRET, bucket };
}

function createRequest(path: string, body: unknown): Request {
  return new Request(`https://example.com${path}`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body)
  });
}

test("signup creates metadata and device entry", async () => {
  const env = createEnv();
  const req = createRequest("/signup", {
    email: "User@example.com",
    password: "supersecret",
    device_id: "dev-1",
    device_name: "Desktop",
    platform: "linux"
  });

  const res = await worker.fetch(req, env as any, {} as any);
  assert.equal(res.status, 200);
  const json = await res.json();
  assert.equal(json.ok, true);
  assert.ok(json.user_id);
  assert.equal(json.email, "user@example.com");

  const metadata = await env.bucket.get(`users/${json.user_id}/metadata.json`);
  assert.ok(metadata, "metadata should be stored");
  const metadataJson = await metadata!.json<any>();
  assert.equal(metadataJson.email, "user@example.com");

  const devices = await env.bucket.get(`users/${json.user_id}/devices.json`);
  assert.ok(devices, "devices should be stored");
  const devicesJson = await devices!.json<any>();
  assert.equal(devicesJson.devices.length, 1);
  assert.equal(devicesJson.devices[0].device_id, "dev-1");
});

test("signup rejects duplicate email", async () => {
  const env = createEnv();
  const first = await worker.fetch(createRequest("/signup", { email: "dup@example.com", password: "password123" }), env as any, {} as any);
  assert.equal(first.status, 200);

  const second = await worker.fetch(createRequest("/signup", { email: "dup@example.com", password: "password456" }), env as any, {} as any);
  assert.equal(second.status, 400);
  const json = await second.json();
  assert.equal(json.error, "email_already_registered");
});

test("login validates password and returns JWT", async () => {
  const env = createEnv();
  const email = "player@example.com";
  await worker.fetch(createRequest("/signup", { email, password: "password789" }), env as any, {} as any);

  const badLogin = await worker.fetch(createRequest("/login", { email, password: "wrong" }), env as any, {} as any);
  assert.equal(badLogin.status, 401);

  const login = await worker.fetch(createRequest("/login", { email, password: "password789" }), env as any, {} as any);
  assert.equal(login.status, 200);
  const json = await login.json();
  assert.ok(json.token);

  const payload = await verifyJwt(env as any, json.token as string);
  assert.ok(payload);
  assert.equal(payload!.user_id.length > 0, true);
});
