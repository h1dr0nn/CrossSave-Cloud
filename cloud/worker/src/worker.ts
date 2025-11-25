import { errorResponse, jsonResponse } from "./utils";
import {
  ensureUserScaffold,
  getSaveObjectKey,
  getUserBaseKey,
  getUserMetadataKey,
  readJson,
  writeJson,
} from "./storage";
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
  generatePresignedGet,
  loadUserMetadata,
  saveUserMetadata,
  UserSaveMetadata
} from "./saveMetadata";
import { requireAccess, requireTurnstile, applySoftRateLimit } from "./middleware";
import {
  validateDeviceId,
  validateEmail,
  validateFileList,
  validateGameId,
  validateSha256,
  validateSizeBytes,
  validateVersionId
} from "./validation";
import { signWorkerToken, verifyWorkerToken } from "./workerToken";

interface Env {
  CROSSSAVE_R2: R2Bucket;
  JWT_SECRET?: string;
  JWT_SECRET_MAIN?: string;
  JWT_SECRET_ROTATED?: string;
  ACCESS_REQUIRED?: string | boolean;
  ACCESS_AUD?: string;
  TURNSTILE_REQUIRED?: string | boolean;
  TURNSTILE_SECRET?: string;
  WORKER_SIGNING_KEY?: string;
  WORKER_SIGNING_KEY_MAIN?: string;
  WORKER_SIGNING_KEY_ROTATED?: string;
}

interface UploadPayload {
  game_id: string;
  version_id: string;
  size_bytes: number;
  sha256: string;
  file_list: string[];
  emulator_id?: string;
  device_id?: string;
}

const SESSION_TTL_SECONDS = 60 * 60 * 24 * 7;
const PRESIGN_TTL_SECONDS = 45;
const WORKER_TOKEN_TTL_SECONDS = 60;
const ROUTE_RATE_LIMITS: Record<string, { limit: number; window: number }> = {
  "/login:POST": { limit: 5, window: 5 },
  "/signup:POST": { limit: 2, window: 10 },
  "/save/upload-url:POST": { limit: 10, window: 10 },
  "/save/notify-upload:POST": { limit: 20, window: 10 },
  "/device/register:POST": { limit: 5, window: 5 },
};

function normalizeEmail(email: string): string {
  return email.trim().toLowerCase();
}

function normalizeDeviceName(deviceName: string | undefined): string {
  const fallback = "Unknown Device";
  if (!deviceName) return fallback;

  const cleaned = deviceName
    .trim()
    .toLowerCase()
    .replace(/\s+/g, " ")
    .replace(/[^a-z0-9 _-]/g, "")
    .trim();

  return cleaned.length ? cleaned : fallback;
}

function normalizePlatform(platform: string | undefined): string {
  if (!platform) return "unknown";
  const cleaned = platform.trim().toLowerCase();
  return cleaned.length ? cleaned : "unknown";
}

async function parseJsonBody(request: Request): Promise<Record<string, unknown> | null> {
  try {
    return await request.json();
  } catch (error) {
    console.error("[worker] invalid JSON body", error);
    return null;
  }
}

function parseDownloadPayload(body: Record<string, unknown>): { game_id: string; version_id: string } | null {
  const gameId = typeof body.game_id === "string" ? body.game_id.trim() : "";
  const versionId = typeof body.version_id === "string" ? body.version_id.trim() : "";

  if (!validateGameId(gameId) || !validateVersionId(versionId)) {
    return null;
  }

  return { game_id: gameId, version_id: versionId };
}

function parseUploadPayload(body: Record<string, unknown>): UploadPayload | null {
  const gameIdRaw = typeof body.game_id === "string" ? body.game_id.trim() : "";
  const versionIdRaw = typeof body.version_id === "string" ? body.version_id.trim() : "";
  const sizeBytes = Number(body.size_bytes);
  const sha256Raw = typeof body.sha256 === "string" ? body.sha256.trim() : "";
  const fileList = validateFileList(body.file_list);
  const emulatorId =
    typeof body.emulator_id === "string" && body.emulator_id.trim().length > 0
      ? body.emulator_id.trim()
      : undefined;
  const deviceId = typeof body.device_id === "string" ? body.device_id.trim() : undefined;

  if (!validateGameId(gameIdRaw) || !validateVersionId(versionIdRaw)) {
    return null;
  }

  if (!validateSha256(sha256Raw) || !validateSizeBytes(sizeBytes)) {
    return null;
  }

  if (!fileList) {
    return null;
  }

  const sortedFiles = [...fileList].sort();

  if (!validateDeviceId(deviceId)) {
    return null;
  }

  return {
    game_id: gameIdRaw,
    version_id: versionIdRaw,
    size_bytes: sizeBytes,
    sha256: sha256Raw,
    file_list: sortedFiles,
    emulator_id: emulatorId,
    device_id: deviceId,
  };
}

async function handleSignup(request: Request, env: Env): Promise<Response> {
  const body = await parseJsonBody(request);
  if (!body) {
    return errorResponse(400, "invalid_json");
  }

  const turnstileCheck = await requireTurnstile(
    typeof body.turnstile_token === "string" ? body.turnstile_token : undefined,
    env
  );
  if (turnstileCheck) {
    return turnstileCheck;
  }

  const email = normalizeEmail(String(body.email || ""));
  const password = String(body.password || "");
  const deviceId = typeof body.device_id === "string" ? body.device_id.trim() : undefined;
  const platform = normalizePlatform(typeof body.platform === "string" ? body.platform : undefined);
  const deviceNameInput = typeof body.device_name === "string" ? body.device_name : undefined;
  const deviceName = normalizeDeviceName(deviceNameInput);

  if (!email || !validateEmail(email)) {
    return errorResponse(400, "invalid_email");
  }
  if (!password || password.length < 8) {
    return errorResponse(400, "weak_password");
  }

  if (!validateDeviceId(deviceId)) {
    return errorResponse(400, "invalid_device_id");
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
      platform,
      last_seen: now,
      device_name: deviceName,
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

  const turnstileCheck = await requireTurnstile(
    typeof body.turnstile_token === "string" ? body.turnstile_token : undefined,
    env
  );
  if (turnstileCheck) {
    return turnstileCheck;
  }

  const email = normalizeEmail(String(body.email || ""));
  const password = String(body.password || "");
  const deviceId = typeof body.device_id === "string" ? body.device_id.trim() : undefined;
  const platform = normalizePlatform(typeof body.platform === "string" ? body.platform : undefined);
  const deviceNameInput = typeof body.device_name === "string" ? body.device_name : undefined;
  const deviceName = normalizeDeviceName(deviceNameInput);

  if (!email || !validateEmail(email) || !password) {
    return errorResponse(401, "invalid_credentials");
  }

  if (!validateDeviceId(deviceId)) {
    return errorResponse(400, "invalid_device_id");
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
      existing.platform = platform;
      existing.device_name = deviceName;
    } else {
      devices.devices.push({
        device_id: deviceId,
        platform,
        last_seen: now,
        device_name: deviceName,
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

  const turnstileCheck = await requireTurnstile(
    typeof body.turnstile_token === "string" ? body.turnstile_token : undefined,
    env
  );
  if (turnstileCheck) {
    return turnstileCheck;
  }

  const payload = parseUploadPayload(body);
  if (!payload) {
    return errorResponse(400, "invalid_payload");
  }

  const deviceId = payload.device_id || auth.device_id;

  try {
    const signed = await generatePresignedPut(
      env.CROSSSAVE_R2,
      auth.user_id,
      payload.game_id,
      payload.version_id,
      payload.size_bytes,
      PRESIGN_TTL_SECONDS
    );

    const now = Math.floor(Date.now() / 1000);
    const workerToken = await signWorkerToken(
      {
        user_id: auth.user_id,
        device_id: deviceId,
        r2_key: signed.key,
        version_id: payload.version_id,
        exp: now + WORKER_TOKEN_TTL_SECONDS,
      },
      env
    );

    return jsonResponse({
      ok: true,
      upload_url: signed.url,
      r2_key: signed.key,
      version_id: payload.version_id,
      worker_token: workerToken,
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

  const turnstileCheck = await requireTurnstile(
    typeof body.turnstile_token === "string" ? body.turnstile_token : undefined,
    env
  );
  if (turnstileCheck) {
    return turnstileCheck;
  }

  const payload = parseUploadPayload(body);
  if (!payload) {
    return errorResponse(400, "invalid_payload");
  }

  const workerToken = typeof body.worker_token === "string" ? body.worker_token : "";
  if (!workerToken) {
    return errorResponse(401, "invalid_worker_token");
  }

  const verified = await verifyWorkerToken(workerToken, env);
  const objectKey = getSaveObjectKey(auth.user_id, payload.game_id, payload.version_id);

  if (!verified || verified.user_id !== auth.user_id || verified.version_id !== payload.version_id || verified.r2_key !== objectKey) {
    return errorResponse(401, "invalid_worker_token");
  }

  if (verified.device_id && payload.device_id && verified.device_id !== payload.device_id) {
    return errorResponse(401, "invalid_worker_token");
  }

  const head = await env.CROSSSAVE_R2.head(objectKey);
  if (!head) {
    return errorResponse(404, "upload_missing");
  }

  if (typeof head.size === "number" && head.size !== payload.size_bytes) {
    return errorResponse(400, "size_mismatch");
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
    emulator_id: payload.emulator_id,
    device_id: payload.device_id || verified.device_id || auth.device_id,
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

async function handleDownloadUrl(
  request: Request,
  env: Env,
  auth: AuthContext
): Promise<Response> {
  const body = await parseJsonBody(request);
  if (!body) {
    return errorResponse(400, "invalid_json");
  }

  const payload = parseDownloadPayload(body);
  if (!payload) {
    return errorResponse(400, "invalid_payload");
  }

  let metadata: UserSaveMetadata;
  try {
    metadata = await loadUserMetadata(env.CROSSSAVE_R2, auth.user_id);
  } catch (error) {
    console.error("[worker] failed to load metadata", error);
    return errorResponse(500, "metadata_load_failed");
  }

  const version = metadata.versions.find(
    (entry) => entry.version_id === payload.version_id && entry.game_id === payload.game_id
  );

  if (!version) {
    return errorResponse(404, "version_not_found");
  }

  const objectKey = getSaveObjectKey(auth.user_id, payload.game_id, payload.version_id);
  const head = await env.CROSSSAVE_R2.head(objectKey);
  if (!head) {
    return errorResponse(404, "object_missing");
  }

  try {
    const signed = await generatePresignedGet(
      env.CROSSSAVE_R2,
      auth.user_id,
      payload.game_id,
      payload.version_id,
      PRESIGN_TTL_SECONDS,
    );

    const response = {
      ok: true,
      download_url: signed.url,
      r2_key: signed.key,
      version_id: version.version_id,
      game_id: version.game_id,
      size_bytes: version.size_bytes,
      sha256: version.sha256,
      file_list: version.file_list,
      emulator_id: version.emulator_id,
      timestamp: version.timestamp,
    };

    try {
      const logKey = `${getUserBaseKey(auth.user_id)}tracking/download_log.json`;
      const existing = (await readJson<Array<Record<string, unknown>>>(env.CROSSSAVE_R2, logKey)) || [];
      const now = Math.floor(Date.now() / 1000);
      existing.push({
        version_id: version.version_id,
        timestamp: now,
        device_id: auth.device_id,
      });
      await writeJson(env.CROSSSAVE_R2, logKey, existing);
    } catch (error) {
      console.warn("[worker] failed to append download tracking", error);
    }

    return jsonResponse(response);
  } catch (error) {
    console.error("[worker] failed to presign download", error);
    return errorResponse(500, "presign_failed");
  }
}

async function handleListSaves(
  request: Request,
  env: Env,
  auth: AuthContext
): Promise<Response> {
  const body = await parseJsonBody(request);
  if (!body) {
    return errorResponse(400, "invalid_json");
  }

  const gameId = typeof body.game_id === "string" ? body.game_id.trim() : "";
  if (!validateGameId(gameId)) {
    return errorResponse(400, "invalid_request");
  }

  const metadataKey = getUserMetadataKey(auth.user_id);
  const head = await env.CROSSSAVE_R2.head(metadataKey);
  if (!head) {
    return jsonResponse({ ok: true, versions: [] });
  }

  let metadata: UserSaveMetadata;
  try {
    metadata = await loadUserMetadata(env.CROSSSAVE_R2, auth.user_id);
  } catch (error) {
    console.error("[worker] failed to parse metadata", error);
    return errorResponse(500, "metadata_corrupted");
  }

  const versions = metadata.versions
    .filter((entry) => entry.game_id === gameId)
    .map((entry) => ({
      version_id: entry.version_id,
      size_bytes: entry.size_bytes,
      timestamp: entry.timestamp,
      device_id: entry.device_id,
      sha256: entry.sha256,
      file_list: Array.isArray(entry.file_list) ? entry.file_list : [],
    }))
    .sort((a, b) => b.timestamp - a.timestamp);

  return jsonResponse({ ok: true, game_id: gameId, versions });
}

async function handleRegisterDevice(request: Request, env: Env, auth: AuthContext): Promise<Response> {
  const body = await parseJsonBody(request);
  if (!body) {
    return errorResponse(400, "invalid_json");
  }

  const deviceId = typeof body.device_id === "string" ? body.device_id.trim() : "";
  const platform = normalizePlatform(typeof body.platform === "string" ? body.platform : undefined);
  const deviceNameInput = typeof body.device_name === "string" ? body.device_name : undefined;
  const deviceName = normalizeDeviceName(deviceNameInput);

  if (!deviceId || !validateDeviceId(deviceId)) {
    return errorResponse(400, "invalid_device_id");
  }

  const devices = await loadUserDevices(env, auth.user_id);
  const now = Math.floor(Date.now() / 1000);
  const existing = devices.devices.find((d) => d.device_id === deviceId);
  if (existing) {
    existing.platform = platform;
    existing.last_seen = now;
    existing.device_name = deviceName;
  } else {
    devices.devices.push({ device_id: deviceId, platform, last_seen: now, device_name: deviceName });
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

  const deviceId = typeof body.device_id === "string" ? body.device_id.trim() : "";
  if (!validateDeviceId(deviceId)) {
    return errorResponse(400, "invalid_device_id");
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

    const accessCheck = await requireAccess(request, env);
    if (accessCheck) {
      return accessCheck;
    }

    const rateKey = `${path}:${request.method.toUpperCase()}`;
    const rateLimit = ROUTE_RATE_LIMITS[rateKey];
    if (rateLimit) {
      const limited = applySoftRateLimit(request, rateKey, rateLimit.limit, rateLimit.window);
      if (limited) {
        return limited;
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

    if (path === "/save/download-url" && request.method === "POST") {
      const auth = await requireAuth(env, request);
      if (!auth) {
        return errorResponse(401, "unauthorized");
      }
      return handleDownloadUrl(request, env, auth);
    }

    if (path === "/save/list" && request.method === "POST") {
      const auth = await requireAuth(env, request);
      if (!auth) {
        return errorResponse(401, "unauthorized");
      }
      return handleListSaves(request, env, auth);
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
