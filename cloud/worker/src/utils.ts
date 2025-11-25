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

export async function parseAuthOrUserId(request: Request): Promise<string | null> {
  const headerUserId = request.headers.get("x-user-id");
  if (headerUserId && headerUserId.trim().length > 0) {
    return headerUserId.trim();
  }

  // TODO: Replace with real authentication and JWT validation once available.
  // For now, fallback to a deterministic dummy user for local development.
  return "local-dev-user";
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
