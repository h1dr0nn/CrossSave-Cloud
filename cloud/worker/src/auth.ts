import { verifyJwt } from "./jwt";

export interface AuthContext {
  user_id: string;
  device_id?: string;
}

interface EnvWithSecret {
  JWT_SECRET?: string;
}

export async function parseAuth(env: EnvWithSecret, request: Request): Promise<AuthContext | null> {
  const authHeader = request.headers.get("authorization") || request.headers.get("Authorization");
  if (!authHeader || !authHeader.toLowerCase().startsWith("bearer ")) {
    return null;
  }

  const token = authHeader.slice(7).trim();
  if (!token) {
    return null;
  }

  const payload = await verifyJwt(env, token);
  if (!payload) {
    return null;
  }

  return { user_id: payload.user_id, device_id: payload.device_id };
}
