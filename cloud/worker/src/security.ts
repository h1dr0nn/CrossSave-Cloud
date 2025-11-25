import { fromBase64, toBase64, utf8ToUint8 } from "./utils";

// Internal flag to decide whether Argon2 is available. Cached after first check.
let argon2ModulePromise: Promise<typeof import("argon2-browser") | null> | null = null;

async function loadArgon2(): Promise<typeof import("argon2-browser") | null> {
  if (!argon2ModulePromise) {
    argon2ModulePromise = import("argon2-browser")
      .then((mod) => mod.default ?? (mod as typeof import("argon2-browser")))
      .catch(() => null);
  }

  return argon2ModulePromise;
}

function getRandomBytes(length: number): Uint8Array {
  const bytes = new Uint8Array(length);
  crypto.getRandomValues(bytes);
  return bytes;
}

async function hashWithArgon2(plain: string): Promise<string | null> {
  const argon2 = await loadArgon2();
  if (!argon2) {
    return null;
  }

  try {
    const salt = getRandomBytes(16);
    const result = await argon2.hash({
      pass: plain,
      salt,
      type: argon2.ArgonType.Argon2id,
      time: 3,
      mem: 2 ** 16,
      hashLen: 32
    });
    if (typeof result.encoded === "string") {
      return result.encoded;
    }
    return null;
  } catch (error) {
    console.warn("[security] Argon2 hashing failed, falling back to PBKDF2", error);
    return null;
  }
}

async function verifyWithArgon2(plain: string, encoded: string): Promise<boolean> {
  const argon2 = await loadArgon2();
  if (!argon2) {
    return false;
  }

  try {
    return argon2.verify({ pass: plain, encoded });
  } catch (error) {
    console.warn("[security] Argon2 verification failed", error);
    return false;
  }
}

async function hashWithPbkdf2(plain: string): Promise<string> {
  const salt = getRandomBytes(16);
  const iterations = 100_000;
  const keyMaterial = await crypto.subtle.importKey(
    "raw",
    utf8ToUint8(plain),
    "PBKDF2",
    false,
    ["deriveBits"]
  );

  const derived = await crypto.subtle.deriveBits(
    {
      name: "PBKDF2",
      hash: "SHA-256",
      iterations,
      salt
    },
    keyMaterial,
    256
  );
  const derivedBytes = new Uint8Array(derived);
  return `pbkdf2$${iterations}$${toBase64(salt)}$${toBase64(derivedBytes)}`;
}

async function verifyWithPbkdf2(plain: string, hash: string): Promise<boolean> {
  const parts = hash.split("$");
  if (parts.length !== 4 || parts[0] !== "pbkdf2") {
    return false;
  }

  const iterations = Number(parts[1]);
  const salt = fromBase64(parts[2]);
  const expected = fromBase64(parts[3]);

  if (!Number.isFinite(iterations) || iterations <= 0) {
    return false;
  }

  const keyMaterial = await crypto.subtle.importKey(
    "raw",
    utf8ToUint8(plain),
    "PBKDF2",
    false,
    ["deriveBits"]
  );

  const derived = await crypto.subtle.deriveBits(
    {
      name: "PBKDF2",
      hash: "SHA-256",
      iterations,
      salt
    },
    keyMaterial,
    expected.length * 8
  );
  const derivedBytes = new Uint8Array(derived);

  if (derivedBytes.length !== expected.length) {
    return false;
  }

  let diff = 0;
  for (let i = 0; i < derivedBytes.length; i++) {
    diff |= derivedBytes[i] ^ expected[i];
  }
  return diff === 0;
}

export async function hashPassword(plain: string): Promise<string> {
  const argonHash = await hashWithArgon2(plain);
  if (argonHash) {
    return argonHash;
  }
  return hashWithPbkdf2(plain);
}

export async function verifyPassword(plain: string, hashed: string): Promise<boolean> {
  if (hashed.startsWith("$argon2")) {
    const argonValid = await verifyWithArgon2(plain, hashed);
    if (argonValid) {
      return true;
    }
    // fallthrough to pbkdf2 verification in case hash is not valid argon format
  }

  if (hashed.startsWith("pbkdf2$")) {
    return verifyWithPbkdf2(plain, hashed);
  }

  return false;
}
