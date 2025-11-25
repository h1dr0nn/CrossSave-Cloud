import { errorResponse } from "./utils";

interface AccessEnv {
  ACCESS_REQUIRED?: string | boolean;
  ACCESS_AUD?: string;
}

interface TurnstileEnv {
  TURNSTILE_REQUIRED?: string | boolean;
  TURNSTILE_SECRET?: string;
}

export async function requireAccess(request: Request, env: AccessEnv): Promise<Response | null> {
  const required = String(env.ACCESS_REQUIRED || "false").toLowerCase() === "true";
  if (!required) {
    return null;
  }

  const assertion = request.headers.get("Cf-Access-Jwt-Assertion");
  if (!assertion) {
    return errorResponse(403, "access_denied");
  }

  try {
    const verify = await fetch("https://identity.cloudflareaccess.com/tokens/verify", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ token: assertion, resource: env.ACCESS_AUD })
    });

    if (!verify.ok) {
      return errorResponse(403, "access_denied");
    }

    const result = await verify.json().catch(() => ({ success: true }));
    if (("success" in result && result.success === false) || result.aud !== env.ACCESS_AUD) {
      return errorResponse(403, "access_denied");
    }

    return null;
  } catch (error) {
    console.error("[access] verification failed", error);
    return errorResponse(403, "access_denied");
  }
}

export async function requireTurnstile(token: string | undefined, env: TurnstileEnv): Promise<Response | null> {
  const required = String(env.TURNSTILE_REQUIRED || "false").toLowerCase() === "true";
  if (!required) {
    return null;
  }

  if (!token) {
    return errorResponse(400, "bot_detected");
  }

  try {
    const verify = await fetch("https://challenges.cloudflare.com/turnstile/v0/siteverify", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ secret: env.TURNSTILE_SECRET, response: token })
    });

    if (!verify.ok) {
      return errorResponse(400, "bot_detected");
    }

    const result = await verify.json().catch(() => ({ success: true }));
    if (result.success === false) {
      return errorResponse(400, "bot_detected");
    }

    return null;
  } catch (error) {
    console.error("[turnstile] verification failed", error);
    return errorResponse(400, "bot_detected");
  }
}

const rateLimitState: Map<string, { count: number; resetAt: number }> = new Map();

function nowMs(): number {
  return Date.now();
}

function parseIp(request: Request): string {
  return (
    request.headers.get("cf-connecting-ip") ||
    request.headers.get("x-forwarded-for") ||
    request.headers.get("x-real-ip") ||
    "unknown"
  );
}

export function applySoftRateLimit(
  request: Request,
  routeKey: string,
  limit: number,
  windowSeconds: number
): Response | null {
  const ip = parseIp(request);
  const key = `${routeKey}:${ip}`;
  const now = nowMs();
  const windowMs = windowSeconds * 1000;
  const entry = rateLimitState.get(key);

  if (!entry || entry.resetAt < now) {
    rateLimitState.set(key, { count: 1, resetAt: now + windowMs });
    return null;
  }

  entry.count += 1;
  if (entry.count > limit) {
    return errorResponse(429, "rate_limited");
  }

  rateLimitState.set(key, entry);
  return null;
}
