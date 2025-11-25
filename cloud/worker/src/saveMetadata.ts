import { getSaveObjectKey, getUserMetadataKey, readJson, writeJson } from "./storage";

export interface VersionMetadataEntry {
  version_id: string;
  game_id: string;
  size_bytes: number;
  sha256: string;
  file_list: string[];
  emulator_id?: string;
  device_id?: string;
  timestamp: number;
}

export interface UserSaveMetadata {
  versions: VersionMetadataEntry[];
  [key: string]: unknown;
}

export function versionIdValid(versionId: string): boolean {
  return /^[A-Za-z0-9_-]{3,64}$/.test(versionId);
}

export async function loadUserMetadata(bucket: R2Bucket, userId: string): Promise<UserSaveMetadata> {
  const key = getUserMetadataKey(userId);
  const existing = (await readJson<Record<string, unknown>>(bucket, key)) || {};
  const versions = Array.isArray((existing as any).versions)
    ? ((existing as any).versions as VersionMetadataEntry[])
    : [];

  const metadata: UserSaveMetadata = {
    ...existing,
    versions: versions.map((entry) => ({
      ...entry,
      file_list: Array.isArray(entry.file_list) ? entry.file_list : [],
    })),
  };

  if (!metadata.versions.length) {
    metadata.versions = [];
  }

  if (metadata.versions.length === 0 && Object.keys(existing).length === 0) {
    await writeJson(bucket, key, metadata);
  }

  return metadata;
}

export async function saveUserMetadata(bucket: R2Bucket, userId: string, metadata: UserSaveMetadata) {
  const sorted = {
    ...metadata,
    versions: [...metadata.versions].sort((a, b) => b.timestamp - a.timestamp),
  };
  await writeJson(bucket, getUserMetadataKey(userId), sorted);
}

export async function generatePresignedPut(
  bucket: R2Bucket,
  userId: string,
  gameId: string,
  versionId: string,
  sizeBytes: number,
  expiresSeconds: number,
): Promise<{ url: string; key: string }> {
  const objectKey = getSaveObjectKey(userId, gameId, versionId);
  const presigned = await bucket.createPresignedUrl({
    method: "PUT",
    key: objectKey,
    expires: expiresSeconds,
    headers: {
      "Content-Length": sizeBytes.toString(),
      "Content-Type": "application/zip",
    },
    contentType: "application/zip",
  } as any);

  return { url: presigned.url.toString(), key: objectKey };
}

export async function generatePresignedGet(
  bucket: R2Bucket,
  userId: string,
  gameId: string,
  versionId: string,
  expiresSeconds: number,
): Promise<{ url: string; key: string }> {
  const objectKey = getSaveObjectKey(userId, gameId, versionId);
  const presigned = await bucket.createPresignedUrl({
    method: "GET",
    key: objectKey,
    expires: expiresSeconds,
  } as any);

  return { url: presigned.url.toString(), key: objectKey };
}
