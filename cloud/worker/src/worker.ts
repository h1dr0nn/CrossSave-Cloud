import { errorResponse, jsonResponse } from "./utils";
import { ensureUserScaffold } from "./storage";
import { hashPassword, verifyPassword } from "./security";
import {
  getUserByEmail,
  loadUserDevices,
  saveUserDevices,
  saveUserMetadata,
  updateLastSeen
} from "./userStore";
import { signJwt } from "./jwt";
import { AuthContext, parseAuth } from "./auth";

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

  const devices = { devices: [] as Array<{ device_id: string; platform: string; last_seen: number }> };
  if (deviceId) {
    devices.devices.push({
      device_id: deviceId,
      platform: platform || "unknown",
      last_seen: now
    });
  }
  await saveUserDevices(env, userId, devices);

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
    const devices = await loadUserDevices(env, user.user_id);
    const existing = devices.devices.find((d) => d.device_id === deviceId);
    if (existing) {
      existing.last_seen = now;
      if (platform) existing.platform = platform;
    } else {
      devices.devices.push({
        device_id: deviceId,
        platform: platform || "unknown",
        last_seen: now
      });
    }
    await saveUserDevices(env, user.user_id, devices);
  }

  const exp = now + SESSION_TTL_SECONDS;
  const token = await signJwt(env, { user_id: user.user_id, device_id: deviceId, exp });

  return jsonResponse({ ok: true, user_id: user.user_id, token, exp, email, device_id: deviceId });
}

async function requireAuth(env: Env, request: Request): Promise<AuthContext | null> {
  const auth = await parseAuth(env, request);
  if (!auth) {
    return null;
  }

  const now = Math.floor(Date.now() / 1000);
  await updateLastSeen(env, auth.user_id, auth.device_id, now);
  return auth;
}

async function handleRegisterDevice(request: Request, env: Env, auth: AuthContext): Promise<Response> {
  const body = await parseJsonBody(request);
  if (!body) {
    return errorResponse(400, "invalid_json");
  }

  const deviceId = typeof body.device_id === "string" ? body.device_id : "";
  const platform = typeof body.platform === "string" && body.platform.trim() ? body.platform : "unknown";

  if (!deviceId) {
    return errorResponse(400, "missing_device_id");
  }

  const devices = await loadUserDevices(env, auth.user_id);
  const now = Math.floor(Date.now() / 1000);
  const existing = devices.devices.find((d) => d.device_id === deviceId);
  if (existing) {
    existing.platform = platform;
    existing.last_seen = now;
  } else {
    devices.devices.push({ device_id: deviceId, platform, last_seen: now });
  }

  await saveUserDevices(env, auth.user_id, devices);
  return jsonResponse({ ok: true, device_id: deviceId });
}

async function handleListDevices(env: Env, auth: AuthContext): Promise<Response> {
  const devices = await loadUserDevices(env, auth.user_id);
  return jsonResponse({ ok: true, devices: devices.devices });
}

async function handleRemoveDevice(request: Request, env: Env, auth: AuthContext): Promise<Response> {
  const body = await parseJsonBody(request);
  if (!body) {
    return errorResponse(400, "invalid_json");
  }

  const deviceId = typeof body.device_id === "string" ? body.device_id : "";
  if (!deviceId) {
    return errorResponse(400, "missing_device_id");
  }

  if (auth.device_id && auth.device_id === deviceId) {
    return errorResponse(400, "cannot_remove_active_device");
  }

  const devices = await loadUserDevices(env, auth.user_id);
  const filtered = devices.devices.filter((device) => device.device_id !== deviceId);
  await saveUserDevices(env, auth.user_id, { devices: filtered });

  return jsonResponse({ ok: true });
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

    if (path === "/device/register" && request.method === "POST") {
      const auth = await requireAuth(env, request);
      if (!auth) {
        return errorResponse(401, "unauthorized");
      }
      return handleRegisterDevice(request, env, auth);
    }

    if (path === "/device/list" && request.method === "GET") {
      const auth = await requireAuth(env, request);
      if (!auth) {
        return errorResponse(401, "unauthorized");
      }
      return handleListDevices(env, auth);
    }

    if (path === "/device/remove" && request.method === "POST") {
      const auth = await requireAuth(env, request);
      if (!auth) {
        return errorResponse(401, "unauthorized");
      }
      return handleRemoveDevice(request, env, auth);
    }

    return errorResponse(404, "Not implemented");
  }
};
