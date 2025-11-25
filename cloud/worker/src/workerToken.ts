import { base64UrlEncode, base64UrlToUint8, utf8ToUint8 } from "./utils";

export interface WorkerTokenPayload {
  user_id: string;
  device_id?: string;
  r2_key: string;
  version_id: string;
  exp: number;
}

interface WorkerTokenEnv {
  WORKER_SIGNING_KEY?: string;
  WORKER_SIGNING_KEY_MAIN?: string;
  WORKER_SIGNING_KEY_ROTATED?: string;
}

function getSigningKeys(env: WorkerTokenEnv): { main: string; rotated?: string } {
  const main = (env.WORKER_SIGNING_KEY_MAIN || env.WORKER_SIGNING_KEY || "").trim();
  const rotated = env.WORKER_SIGNING_KEY_ROTATED?.trim();
  if (!main) {
    throw new Error("Worker signing key is not configured");
  }
  return { main, rotated: rotated && rotated.length > 0 ? rotated : undefined };
}

async function createKey(secret: string): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    "raw",
    utf8ToUint8(secret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign", "verify"]
  );
}

async function hmacSha256(key: CryptoKey, data: Uint8Array): Promise<Uint8Array> {
  const signature = await crypto.subtle.sign("HMAC", key, data);
  return new Uint8Array(signature);
}

export async function signWorkerToken(payload: WorkerTokenPayload, env: WorkerTokenEnv): Promise<string> {
  const { main } = getSigningKeys(env);
  const encodedPayload = base64UrlEncode(JSON.stringify(payload));
  const key = await createKey(main);
  const signingInput = utf8ToUint8(encodedPayload);
  const signature = await hmacSha256(key, signingInput);
  const encodedSignature = base64UrlEncode(signature);
  return `${encodedPayload}.${encodedSignature}`;
}

async function verifyWithKey(secret: string, encodedPayload: string, encodedSignature: string): Promise<boolean> {
  const key = await createKey(secret);
  const signingInput = utf8ToUint8(encodedPayload);
  const signatureBytes = base64UrlToUint8(encodedSignature);
  return crypto.subtle.verify("HMAC", key, signatureBytes, signingInput);
}

export async function verifyWorkerToken(token: string, env: WorkerTokenEnv): Promise<WorkerTokenPayload | null> {
  try {
    const { main, rotated } = getSigningKeys(env);
    const parts = token.split(".");
    if (parts.length !== 2) {
      return null;
    }

    const [encodedPayload, encodedSignature] = parts;
    const mainValid = await verifyWithKey(main, encodedPayload, encodedSignature);
    const rotatedValid = !mainValid && rotated
      ? await verifyWithKey(rotated, encodedPayload, encodedSignature)
      : mainValid;

    if (!mainValid && !rotatedValid) {
      return null;
    }

    const payload: WorkerTokenPayload = JSON.parse(new TextDecoder().decode(base64UrlToUint8(encodedPayload)));
    if (!payload.exp || !payload.user_id || !payload.r2_key || !payload.version_id) {
      return null;
    }

    const now = Math.floor(Date.now() / 1000);
    if (payload.exp < now) {
      return null;
    }

    return payload;
  } catch (error) {
    console.error("[worker-token] verification failed", error);
    return null;
  }
}
