import { base64UrlEncode, base64UrlToUint8, utf8ToUint8 } from "./utils";

interface EnvWithSecret {
  JWT_SECRET?: string;
  JWT_SECRET_MAIN?: string;
  JWT_SECRET_ROTATED?: string;
}

type JwtPayload = { user_id: string; device_id?: string; exp: number };

type JwtHeader = { alg: "HS256"; typ: "JWT" };

function getSecrets(env: EnvWithSecret): { main: string; rotated?: string } {
  const main = (env.JWT_SECRET_MAIN || env.JWT_SECRET || "").trim();
  const rotated = env.JWT_SECRET_ROTATED?.trim();
  if (!main) {
    throw new Error("JWT secret is not configured");
  }
  return { main, rotated: rotated && rotated.length > 0 ? rotated : undefined };
}

async function hmacSha256(key: CryptoKey, data: Uint8Array): Promise<Uint8Array> {
  const signature = await crypto.subtle.sign("HMAC", key, data);
  return new Uint8Array(signature);
}

async function createHmacKey(secret: string): Promise<CryptoKey> {
  return crypto.subtle.importKey(
    "raw",
    utf8ToUint8(secret),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign", "verify"]
  );
}

export async function signJwt(env: EnvWithSecret, payload: JwtPayload): Promise<string> {
  const header: JwtHeader = { alg: "HS256", typ: "JWT" };
  const encodedHeader = base64UrlEncode(JSON.stringify(header));
  const encodedPayload = base64UrlEncode(JSON.stringify(payload));
  const { main: secret } = getSecrets(env);

  const key = await createHmacKey(secret);
  const signingInput = utf8ToUint8(`${encodedHeader}.${encodedPayload}`);
  const signatureBytes = await hmacSha256(key, signingInput);
  const encodedSignature = base64UrlEncode(signatureBytes);

  return `${encodedHeader}.${encodedPayload}.${encodedSignature}`;
}

async function verifyWithSecret(secret: string, encodedHeader: string, encodedPayload: string, encodedSignature: string) {
  const key = await createHmacKey(secret);
  const signingInput = utf8ToUint8(`${encodedHeader}.${encodedPayload}`);
  const signatureBytes = base64UrlToUint8(encodedSignature);
  const valid = await crypto.subtle.verify("HMAC", key, signatureBytes, signingInput);
  return valid;
}

export async function verifyJwt(env: EnvWithSecret, token: string): Promise<JwtPayload | null> {
  try {
    const { main, rotated } = getSecrets(env);
    const parts = token.split(".");
    if (parts.length !== 3) {
      return null;
    }

    const [encodedHeader, encodedPayload, encodedSignature] = parts;
    const headerJson = JSON.parse(new TextDecoder().decode(base64UrlToUint8(encodedHeader)));
    if (headerJson.alg !== "HS256" || headerJson.typ !== "JWT") {
      return null;
    }

    const payloadBytes = base64UrlToUint8(encodedPayload);
    const payload: JwtPayload = JSON.parse(new TextDecoder().decode(payloadBytes));
    if (!payload.exp || typeof payload.user_id !== "string") {
      return null;
    }

    const mainValid = await verifyWithSecret(main, encodedHeader, encodedPayload, encodedSignature);
    const rotatedValid = !mainValid && rotated
      ? await verifyWithSecret(rotated, encodedHeader, encodedPayload, encodedSignature)
      : mainValid;

    if (!mainValid && !rotatedValid) {
      return null;
    }

    const now = Math.floor(Date.now() / 1000);
    if (payload.exp < now) {
      return null;
    }

    return payload;
  } catch (error) {
    console.error("[jwt] verify failed", error);
    return null;
  }
}

export type { JwtPayload };
