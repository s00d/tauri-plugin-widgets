export type { PresetDef } from "./types";

import type { PresetDef } from "./types";
import { weather } from "./weather";
import { fitness } from "./fitness";
import { tasks } from "./tasks";
import { revenue } from "./revenue";
import { music } from "./music";
import { crypto } from "./crypto";
import { system } from "./system";
import { calendar } from "./calendar";
import { social } from "./social";
import { quote } from "./quote";
import { battery } from "./battery";
import { countdown } from "./countdown";
import { analogClock } from "./analogClock";
import { trafficLight } from "./trafficLight";
import { liveMetrics } from "./liveMetrics";
import { calculator } from "./calculator";
import { userProfile } from "./userProfile";
import { darkModeDemo } from "./darkModeDemo";

export const PRESETS: Record<string, PresetDef> = {
  weather,
  fitness,
  tasks,
  revenue,
  music,
  crypto,
  system,
  calendar,
  social,
  quote,
  battery,
  countdown,
  analogClock,
  trafficLight,
  liveMetrics,
  calculator,
  userProfile,
  darkModeDemo,
};
