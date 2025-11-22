// Import real emulator SVG icons
import RetroArchIcon from "../icons/retroarch-svgrepo-com.svg?raw";
import AetherSX2Icon from "../icons/AetherSX2 - icon.svg?raw";
import DolphinIcon from "../icons/Dolphin - icon.svg?raw";
import DuckstationIcon from "../icons/Duckstation.svg?raw";
import PPSSPPIcon from "../icons/ppsspp.svg?raw";
import DeSmuMEIcon from "../icons/desmume.svg?raw";

/**
 * Returns the appropriate SVG icon for each emulator based on its ID
 */
export function getEmulatorIcon(emulatorId: string): string {
  const icons: Record<string, string> = {
    retroarch: RetroArchIcon,
    aethersx2: AetherSX2Icon,
    dolphin: DolphinIcon,
    duckstation: DuckstationIcon,
    ppsspp: PPSSPPIcon,
    drastic: DeSmuMEIcon, // Using DeSmuME icon for DraStic (both are DS emulators)
  };

  return icons[emulatorId] || RetroArchIcon; // Fallback to RetroArch icon
}
