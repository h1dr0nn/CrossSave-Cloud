import {
  getUserDevicesKey,
  getUserMetadataKey,
  readJson,
  writeJson
} from "./storage";

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
  name?: string;
  platform?: string;
  last_seen: number;
  created_at: number;
}

export interface DevicesEntry {
  user_id: string;
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

export async function getDevices(env: { CROSSSAVE_R2: R2Bucket }, userId: string): Promise<DevicesEntry> {
  const existing = await readJson<DevicesEntry>(env.CROSSSAVE_R2, getUserDevicesKey(userId));
  if (existing) {
    return existing;
  }
  return { user_id: userId, devices: [] };
}

export async function saveDevices(env: { CROSSSAVE_R2: R2Bucket }, devices: DevicesEntry): Promise<void> {
  await writeJson(env.CROSSSAVE_R2, getUserDevicesKey(devices.user_id), devices);
}
