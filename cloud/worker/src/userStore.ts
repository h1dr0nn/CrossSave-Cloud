import { getUserDevicesKey, getUserMetadataKey, readJson, writeJson } from "./storage";

export interface UserMetadata {
  user_id: string;
  email: string;
  password_hash: string;
  created_at: number;
  updated_at: number;
  devices?: number;
}

export interface DeviceInfo {
  device_id: string;
  platform: string;
  last_seen: number;
}

export interface DevicesEntry {
  devices: DeviceInfo[];
}

interface EmailIndex {
  by_email: Record<string, string>;
}

const EMAIL_INDEX_KEY = "users/index.json";

function defaultEmailIndex(): EmailIndex {
  return { by_email: {} };
}

export async function getUserByEmail(env: { CROSSSAVE_R2: R2Bucket }, email: string): Promise<UserMetadata | null> {
  const index = (await readJson<EmailIndex>(env.CROSSSAVE_R2, EMAIL_INDEX_KEY)) ?? defaultEmailIndex();
  const userId = index.by_email[email];
  if (!userId) {
    return null;
  }
  return getUserById(env, userId);
}

export async function getUserById(env: { CROSSSAVE_R2: R2Bucket }, userId: string): Promise<UserMetadata | null> {
  return readJson<UserMetadata>(env.CROSSSAVE_R2, getUserMetadataKey(userId));
}

async function updateEmailIndex(
  bucket: R2Bucket,
  email: string,
  userId: string,
  attempt = 0
): Promise<void> {
  const maxAttempts = 3;
  const index = (await readJson<EmailIndex>(bucket, EMAIL_INDEX_KEY)) ?? defaultEmailIndex();
  index.by_email[email] = userId;
  try {
    await writeJson(bucket, EMAIL_INDEX_KEY, index);
  } catch (error) {
    if (attempt + 1 >= maxAttempts) {
      throw error;
    }
    await updateEmailIndex(bucket, email, userId, attempt + 1);
  }
}

export async function saveUserMetadata(env: { CROSSSAVE_R2: R2Bucket }, user: UserMetadata): Promise<void> {
  await writeJson(env.CROSSSAVE_R2, getUserMetadataKey(user.user_id), user);
  await updateEmailIndex(env.CROSSSAVE_R2, user.email, user.user_id);
}

export async function loadUserDevices(env: { CROSSSAVE_R2: R2Bucket }, userId: string): Promise<DevicesEntry> {
  const existing = await readJson<DevicesEntry>(env.CROSSSAVE_R2, getUserDevicesKey(userId));
  if (existing) {
    return existing;
  }

  const empty = { devices: [] as DeviceInfo[] };
  await writeJson(env.CROSSSAVE_R2, getUserDevicesKey(userId), empty);
  return empty;
}

export async function saveUserDevices(env: { CROSSSAVE_R2: R2Bucket }, userId: string, devices: DevicesEntry): Promise<void> {
  await writeJson(env.CROSSSAVE_R2, getUserDevicesKey(userId), devices);
}

export async function updateLastSeen(
  env: { CROSSSAVE_R2: R2Bucket },
  userId: string,
  deviceId: string | undefined,
  timestamp: number,
  platform = "unknown"
): Promise<void> {
  if (!deviceId) {
    return;
  }

  const devices = await loadUserDevices(env, userId);
  const existing = devices.devices.find((device) => device.device_id === deviceId);
  if (existing) {
    existing.last_seen = timestamp;
  } else {
    devices.devices.push({ device_id: deviceId, platform, last_seen: timestamp });
  }

  await saveUserDevices(env, userId, devices);
}
