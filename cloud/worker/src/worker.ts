import { errorResponse, jsonResponse } from "./utils";
import { ensureUserScaffold } from "./storage";
import { hashPassword, verifyPassword } from "./security";
import { getDevices, getUserByEmail, saveDevices, saveUserMetadata } from "./userStore";
import { signJwt } from "./jwt";

interface Env {
  CROSSSAVE_R2: R2Bucket;
  JWT_SECRET?: string;
}

const SESSION_TTL_SECONDS = 60 * 60 * 24 * 7;

function normalizeEmail(email: string): string {
  return email.trim().toLowerCase();
}

function isValidEmail(email: string): boolean {
  return /.+@.+\..+/.test(email);
}

async function parseJsonBody(request: Request): Promise<Record<string, unknown> | null> {
  try {
    return await request.json();
  } catch (error) {
    console.error("[worker] invalid JSON body", error);
    return null;
  }
}

async function handleSignup(request: Request, env: Env): Promise<Response> {
  const body = await parseJsonBody(request);
  if (!body) {
    return errorResponse(400, "invalid_json");
  }

  const email = normalizeEmail(String(body.email || ""));
  const password = String(body.password || "");
  const deviceId = typeof body.device_id === "string" ? body.device_id : undefined;
  const deviceName = typeof body.device_name === "string" ? body.device_name : undefined;
  const platform = typeof body.platform === "string" ? body.platform : undefined;

  if (!email || !isValidEmail(email)) {
    return errorResponse(400, "invalid_email");
  }
  if (!password || password.length < 8) {
    return errorResponse(400, "weak_password");
  }

  const existing = await getUserByEmail(env, email);
  if (existing) {
    return errorResponse(400, "email_already_registered");
  }

  const userId = crypto.randomUUID();
  const passwordHash = await hashPassword(password);
  const now = Math.floor(Date.now() / 1000);

  await saveUserMetadata(env, {
    user_id: userId,
    email,
    password_hash: passwordHash,
    created_at: now,
    updated_at: now,
    devices: deviceId ? 1 : 0
  });

  const devices = { user_id: userId, devices: [] as Array<{ device_id: string; name?: string; platform?: string; last_seen: number; created_at: number }> };
  if (deviceId) {
    devices.devices.push({
      device_id: deviceId,
      name: deviceName,
      platform,
      last_seen: now,
      created_at: now
    });
  }
  await saveDevices(env, devices);

  const exp = now + SESSION_TTL_SECONDS;
  const token = await signJwt(env, { user_id: userId, device_id: deviceId, exp });

  return jsonResponse({ ok: true, user_id: userId, token, exp, device_id: deviceId, email });
}

async function handleLogin(request: Request, env: Env): Promise<Response> {
  const body = await parseJsonBody(request);
  if (!body) {
    return errorResponse(400, "invalid_json");
  }

  const email = normalizeEmail(String(body.email || ""));
  const password = String(body.password || "");
  const deviceId = typeof body.device_id === "string" ? body.device_id : undefined;
  const deviceName = typeof body.device_name === "string" ? body.device_name : undefined;
  const platform = typeof body.platform === "string" ? body.platform : undefined;

  if (!email || !isValidEmail(email) || !password) {
    return errorResponse(401, "invalid_credentials");
  }

  const user = await getUserByEmail(env, email);
  if (!user) {
    return errorResponse(401, "invalid_credentials");
  }

  const valid = await verifyPassword(password, user.password_hash);
  if (!valid) {
    return errorResponse(401, "invalid_credentials");
  }

  const now = Math.floor(Date.now() / 1000);
  if (deviceId) {
    const devices = await getDevices(env, user.user_id);
    const existing = devices.devices.find((d) => d.device_id === deviceId);
    if (existing) {
      existing.last_seen = now;
      if (deviceName) existing.name = deviceName;
      if (platform) existing.platform = platform;
    } else {
      devices.devices.push({
        device_id: deviceId,
        name: deviceName,
        platform,
        created_at: now,
        last_seen: now
      });
    }
    await saveDevices(env, devices);
  }

  const exp = now + SESSION_TTL_SECONDS;
  const token = await signJwt(env, { user_id: user.user_id, device_id: deviceId, exp });

  return jsonResponse({ ok: true, user_id: user.user_id, token, exp, email, device_id: deviceId });
}

export default {
  async fetch(request: Request, env: Env, _ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);
    const path = url.pathname;

    console.log(`[worker] ${request.method} ${path}`);

    if (path === "/health") {
      try {
        const userId = "health-check";
        await ensureUserScaffold(env.CROSSSAVE_R2, userId);
        return jsonResponse({ ok: true, r2: "ready" });
      } catch (error) {
        console.error("[worker] /health failed", error);
        return errorResponse(500, "R2 not ready");
      }
    }

    if (path === "/signup" && request.method === "POST") {
      return handleSignup(request, env);
    }

    if (path === "/login" && request.method === "POST") {
      return handleLogin(request, env);
    }

    return errorResponse(404, "Not implemented");
  }
};
