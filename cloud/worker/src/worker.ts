import { errorResponse, jsonResponse } from "./utils";
import { ensureUserScaffold, getSaveObjectKey } from "./storage";
import { hashPassword, verifyPassword } from "./security";
import {
  getUserByEmail,
  loadUserDevices,
  saveUserDevices,
  saveUserMetadata as saveAccountMetadata,
  updateLastSeen
} from "./userStore";
import { signJwt } from "./jwt";
import { AuthContext, parseAuth } from "./auth";
import {
  generatePresignedPut,
  loadUserMetadata,
  saveUserMetadata,
  UserSaveMetadata,
  versionIdValid
} from "./saveMetadata";

interface Env {
  CROSSSAVE_R2: R2Bucket;
  JWT_SECRET?: string;
}

interface UploadPayload {
  game_id: string;
  version_id: string;
  size_bytes: number;
  sha256: string;
  file_list: string[];
  device_id?: string;
}

const SESSION_TTL_SECONDS = 60 * 60 * 24 * 7;

function normalizeEmail(email: string): string {
  return email.trim().toLowerCase();
}

function normalizeDeviceName(deviceName: string | undefined): string {
  if (!deviceName) return "Unknown device";
  const collapsed = deviceName.trim().replace(/\s+/g, " ");
  return collapsed.length ? collapsed : "Unknown device";
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

function parseUploadPayload(body: Record<string, unknown>): UploadPayload | null {
  const gameId = typeof body.game_id === "string" ? body.game_id.trim() : "";
  const versionId = typeof body.version_id === "string" ? body.version_id.trim() : "";
  const sizeBytes = Number(body.size_bytes);
  const sha256 = typeof body.sha256 === "string" ? body.sha256.trim() : "";
  const fileList = Array.isArray(body.file_list)
    ? body.file_list.map((f) => String(f)).filter((f) => f.trim().length > 0)
    : [];
  const deviceId = typeof body.device_id === "string" ? body.device_id.trim() : undefined;

  fileList.sort();

  if (!gameId || !versionIdValid(versionId) || !sha256) {
    return null;
  }

  if (!Number.isFinite(sizeBytes) || sizeBytes < 0) {
    return null;
  }

  return {
    game_id: gameId,
    version_id: versionId,
    size_bytes: sizeBytes,
    sha256,
    file_list: fileList,
    device_id: deviceId,
  };
}

async function handleSignup(request: Request, env: Env): Promise<Response> {
  const body = await parseJsonBody(request);
  if (!body) {
    return errorResponse(400, "invalid_json");
  }

  const email = normalizeEmail(String(body.email || ""));
  const password = String(body.password || "");
  const deviceId = typeof body.device_id === "string" ? body.device_id.trim() : undefined;
  const platform = typeof body.platform === "string" ? body.platform.trim() : undefined;
  const deviceNameInput = typeof body.device_name === "string" ? body.device_name : undefined;
  const deviceName =
    deviceNameInput && deviceNameInput.trim().length > 0
      ? normalizeDeviceName(deviceNameInput)
      : undefined;

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

  await saveAccountMetadata(env, {
    user_id: userId,
    email,
    password_hash: passwordHash,
    created_at: now,
    updated_at: now,
    devices: deviceId ? 1 : 0
  });

  const devices = {
    devices: [] as Array<{ device_id: string; platform: string; last_seen: number; device_name: string }>,
  };
  if (deviceId) {
    devices.devices.push({
      device_id: deviceId,
      platform: platform || "unknown",
      last_seen: now,
      device_name: deviceName || "Unknown device",
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
  const deviceId = typeof body.device_id === "string" ? body.device_id.trim() : undefined;
  const platform = typeof body.platform === "string" ? body.platform.trim() : undefined;
  const deviceNameInput = typeof body.device_name === "string" ? body.device_name : undefined;
  const deviceName =
    deviceNameInput && deviceNameInput.trim().length > 0
      ? normalizeDeviceName(deviceNameInput)
      : undefined;

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
      if (deviceName) existing.device_name = deviceName;
    } else {
      devices.devices.push({
        device_id: deviceId,
        platform: platform || "unknown",
        last_seen: now,
        device_name: deviceName || "Unknown device",
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

async function handleUploadUrl(
  request: Request,
  env: Env,
  auth: AuthContext
): Promise<Response> {
  const body = await parseJsonBody(request);
  if (!body) {
    return errorResponse(400, "invalid_json");
  }

  const payload = parseUploadPayload(body);
  if (!payload) {
    return errorResponse(400, "invalid_payload");
  }

  try {
    const signed = await generatePresignedPut(
      env.CROSSSAVE_R2,
      auth.user_id,
      payload.game_id,
      payload.version_id,
      60 * 15
    );

    return jsonResponse({
      ok: true,
      upload_url: signed.url,
      r2_key: signed.key,
      version_id: payload.version_id,
    });
  } catch (error) {
    console.error("[worker] failed to generate presigned url", error);
    return errorResponse(500, "presign_failed");
  }
}

async function handleNotifyUpload(
  request: Request,
  env: Env,
  auth: AuthContext
): Promise<Response> {
  const body = await parseJsonBody(request);
  if (!body) {
    return errorResponse(400, "invalid_json");
  }

  const payload = parseUploadPayload(body);
  if (!payload) {
    return errorResponse(400, "invalid_payload");
  }

  const objectKey = getSaveObjectKey(auth.user_id, payload.game_id, payload.version_id);
  const head = await env.CROSSSAVE_R2.head(objectKey);
  if (!head) {
    return errorResponse(404, "upload_missing");
  }

  let metadata: UserSaveMetadata;
  try {
    metadata = await loadUserMetadata(env.CROSSSAVE_R2, auth.user_id);
  } catch (error) {
    console.error("[worker] failed to load metadata", error);
    return errorResponse(500, "metadata_load_failed");
  }

  const now = Math.floor(Date.now() / 1000);
  const entry = {
    version_id: payload.version_id,
    game_id: payload.game_id,
    size_bytes: payload.size_bytes,
    sha256: payload.sha256,
    file_list: payload.file_list,
    device_id: payload.device_id || auth.device_id,
    timestamp: now,
  };

  const filtered = metadata.versions.filter((v) => v.version_id !== payload.version_id);
  metadata.versions = [entry, ...filtered];

  try {
    await saveUserMetadata(env.CROSSSAVE_R2, auth.user_id, metadata);
  } catch (error) {
    console.error("[worker] failed to save metadata", error);
    return errorResponse(500, "metadata_save_failed");
  }

  return jsonResponse({ ok: true });
}

async function handleRegisterDevice(request: Request, env: Env, auth: AuthContext): Promise<Response> {
  const body = await parseJsonBody(request);
  if (!body) {
    return errorResponse(400, "invalid_json");
  }

  const deviceId = typeof body.device_id === "string" ? body.device_id.trim() : "";
  const platform = typeof body.platform === "string" && body.platform.trim() ? body.platform.trim() : "unknown";
  const deviceNameInput = typeof body.device_name === "string" ? body.device_name : undefined;
  const deviceName =
    deviceNameInput && deviceNameInput.trim().length > 0
      ? normalizeDeviceName(deviceNameInput)
      : undefined;

  if (!deviceId) {
    return errorResponse(400, "missing_device_id");
  }

  const devices = await loadUserDevices(env, auth.user_id);
  const now = Math.floor(Date.now() / 1000);
  const existing = devices.devices.find((d) => d.device_id === deviceId);
  if (existing) {
    existing.platform = platform;
    existing.last_seen = now;
    if (deviceName) existing.device_name = deviceName;
  } else {
    devices.devices.push({ device_id: deviceId, platform, last_seen: now, device_name: deviceName || "Unknown device" });
  }

  await saveUserDevices(env, auth.user_id, devices);
  const device = devices.devices.find((d) => d.device_id === deviceId)!;
  return jsonResponse({ ok: true, device });
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
  if (filtered.length === devices.devices.length) {
    return errorResponse(404, "device_not_found");
  }
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

    if (path === "/save/upload-url" && request.method === "POST") {
      const auth = await requireAuth(env, request);
      if (!auth) {
        return errorResponse(401, "unauthorized");
      }
      return handleUploadUrl(request, env, auth);
    }

    if (path === "/save/notify-upload" && request.method === "POST") {
      const auth = await requireAuth(env, request);
      if (!auth) {
        return errorResponse(401, "unauthorized");
      }
      return handleNotifyUpload(request, env, auth);
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
