const MAX_FILE_COUNT = 200;
const MAX_SIZE_BYTES = 2 * 1024 * 1024 * 1024; // 2 GB

export function validateEmail(email: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email.trim());
}

export function validateDeviceId(deviceId: string | undefined): boolean {
  if (deviceId === undefined) return true;
  return /^[A-Za-z0-9_-]{3,128}$/.test(deviceId.trim());
}

export function validateGameId(gameId: string): boolean {
  return /^[A-Za-z0-9_.-]{1,128}$/.test(gameId.trim());
}

export function validateVersionId(versionId: string): boolean {
  return /^[A-Za-z0-9_-]{3,64}$/.test(versionId.trim());
}

export function validateSha256(hash: string): boolean {
  return /^[a-fA-F0-9]{64}$/.test(hash.trim());
}

export function validateFileList(list: unknown): string[] | null {
  if (!Array.isArray(list)) {
    return null;
  }

  if (list.length > MAX_FILE_COUNT) {
    return null;
  }

  const cleaned: string[] = [];
  for (const entry of list) {
    const value = String(entry).trim();
    if (!value) {
      return null;
    }
    cleaned.push(value);
  }

  return cleaned;
}

export function validateSizeBytes(sizeBytes: number): boolean {
  return Number.isFinite(sizeBytes) && sizeBytes > 0 && sizeBytes <= MAX_SIZE_BYTES;
}
