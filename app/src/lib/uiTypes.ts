export interface GameEntry {
  id: string;
  emulatorId: string;
  name: string;
  lastModified: string;
  icon: "default" | "console" | "spark" | "disc";
}
