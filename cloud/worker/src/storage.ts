export function getUserBaseKey(userId: string): string {
  return `users/${userId}/`;
}

export function getUserMetadataKey(userId: string): string {
  return `${getUserBaseKey(userId)}metadata.json`;
}

export function getUserDevicesKey(userId: string): string {
  return `${getUserBaseKey(userId)}devices.json`;
}

export function getSaveObjectKey(userId: string, gameId: string, versionId: string): string {
  return `${getUserBaseKey(userId)}saves/${gameId}/${versionId}.zip`;
}

export async function readJson<T>(bucket: R2Bucket, key: string): Promise<T | null> {
  const object = await bucket.get(key);
  if (!object) {
    return null;
  }

  return object.json<T>();
}

export async function writeJson(bucket: R2Bucket, key: string, value: unknown): Promise<void> {
  await bucket.put(key, JSON.stringify(value), {
    httpMetadata: {
      contentType: "application/json"
    }
  });
}

export async function ensureUserScaffold(bucket: R2Bucket, userId: string): Promise<void> {
  const metadataKey = getUserMetadataKey(userId);
  const devicesKey = getUserDevicesKey(userId);

  const [metadataHead, devicesHead] = await Promise.all([
    bucket.head(metadataKey),
    bucket.head(devicesKey)
  ]);

  if (!metadataHead) {
    const now = Math.floor(Date.now() / 1000);
    const metadata = {
      user_id: userId,
      email: `${userId}@placeholder.local`,
      password_hash: "",
      created_at: now,
      updated_at: now,
      devices: 0
    };
    await writeJson(bucket, metadataKey, metadata);
  }

  if (!devicesHead) {
    const devices = {
      user_id: userId,
      devices: [] as Array<unknown>
    };
    await writeJson(bucket, devicesKey, devices);
  }
}
