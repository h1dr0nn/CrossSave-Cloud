export function jsonResponse(data: unknown, init: ResponseInit = {}): Response {
  const headers = new Headers(init.headers || {});
  if (!headers.has("content-type")) {
    headers.set("content-type", "application/json");
  }

  return new Response(JSON.stringify(data), { ...init, headers });
}

export function errorResponse(status: number, message: string): Response {
  return jsonResponse({ error: message }, { status });
}

export async function computeSha256(input: ArrayBuffer | Uint8Array | string): Promise<string> {
  let data: ArrayBuffer;
  if (typeof input === "string") {
    data = new TextEncoder().encode(input).buffer;
  } else if (input instanceof Uint8Array) {
    data = input.buffer.slice(input.byteOffset, input.byteOffset + input.byteLength);
  } else {
    data = input;
  }

  const hashBuffer = await crypto.subtle.digest("SHA-256", data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");
}

export function utf8ToUint8(input: string): Uint8Array {
  return new TextEncoder().encode(input);
}

function toBase64Internal(data: Uint8Array): string {
  if (typeof btoa !== "undefined") {
    let binary = "";
    for (let i = 0; i < data.length; i++) {
      binary += String.fromCharCode(data[i]);
    }
    return btoa(binary);
  }
  // Node.js fallback
  const bufferCtor = (globalThis as any).Buffer;
  if (bufferCtor) {
    return bufferCtor.from(data).toString("base64");
  }
  throw new Error("No base64 encoder available");
}

export function toBase64(data: Uint8Array): string {
  return toBase64Internal(data);
}

export function fromBase64(input: string): Uint8Array {
  if (typeof atob !== "undefined") {
    const binary = atob(input);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
  }
  const bufferCtor = (globalThis as any).Buffer;
  if (bufferCtor) {
    return new Uint8Array(bufferCtor.from(input, "base64"));
  }
  throw new Error("No base64 decoder available");
}

export function base64UrlEncode(data: string | Uint8Array): string {
  const bytes = typeof data === "string" ? utf8ToUint8(data) : data;
  return toBase64Internal(bytes).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/g, "");
}

export function toBase64Url(data: Uint8Array): string {
  return base64UrlEncode(data);
}

export function base64UrlToUint8(input: string): Uint8Array {
  let normalized = input.replace(/-/g, "+").replace(/_/g, "/");
  const padding = normalized.length % 4;
  if (padding === 2) normalized += "==";
  else if (padding === 3) normalized += "=";
  else if (padding !== 0) throw new Error("Invalid base64url input");

  return fromBase64(normalized);
}

export function randomBase64Url(length = 32): string {
  const bytes = new Uint8Array(length);
  crypto.getRandomValues(bytes);
  return toBase64Url(bytes);
}
