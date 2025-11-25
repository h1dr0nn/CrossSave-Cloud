export function getIconVariant(id: string): "default" | "console" | "spark" | "disc" {
  let hash = 0;
  for (let i = 0; i < id.length; i++) {
    hash = id.charCodeAt(i) + ((hash << 5) - hash);
  }
  const variants = ["default", "console", "spark", "disc"];
  return variants[Math.abs(hash) % variants.length] as any;
}

export function extractGameName(path: string): string {
  if (!path) return "Unknown Game";
  // Extract filename from path
  const filename = path.split(/[/\\]/).pop() || path;
  // Remove extension
  const name = filename.replace(/\.[^/.]+$/, "");
  // Clean up underscores and dashes
  const clean = name.replace(/[_-]+/g, " ").trim();

  // Capitalize first letter of each word
  return clean
    .split(" ")
    .map((chunk) => chunk.charAt(0).toUpperCase() + chunk.slice(1))
    .join(" ");
}

export function deriveEmulatorId(gameId: string): string {
  // Default to retroarch for now
  // TODO: In the future, derive from game path or allow user to select
  return "retroarch";
}
