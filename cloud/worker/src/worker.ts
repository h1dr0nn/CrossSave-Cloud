import { errorResponse, jsonResponse } from "./utils";
import { ensureUserScaffold } from "./storage";

interface Env {
  CROSSSAVE_R2: R2Bucket;
}

export default {
  async fetch(request: Request, env: Env, _ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url);
    const path = url.pathname;

    console.log(`[worker] ${request.method} ${path}`);

    if (path === "/health") {
      try {
        const userId = "health-check";
        await ensureUserScaffold(env.CROSSSAVE_R2, userId);
        return jsonResponse({ ok: true, r2: "ready" });
      } catch (error) {
        console.error("[worker] /health failed", error);
        return errorResponse(500, "R2 not ready");
      }
    }

    return errorResponse(404, "Not implemented");
  }
};
