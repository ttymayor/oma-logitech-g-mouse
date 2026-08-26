#!/usr/bin/env bun
// superstrike-daemon.ts — HID++ 2.0 poller & controller for Logitech G PRO X2 SUPERSTRIKE.

import { readFileSync, readdirSync, openSync, readSync, writeSync, closeSync, constants, writeFileSync } from "node:fs";
import { join } from "node:path";

// ── Constants ────────────────────────────────────────────────────────────

const VID = 0x046d;

// HID++ 2.0 framing
const REPORT_LONG = 0x11;
const LONG_LEN = 20;
const SWID = 0x07;

// Feature IDs
const FEAT_NAME = 0x0005;
const FEAT_UNIFIED_BATTERY = 0x1004;
const FEAT_EXT_DPI = 0x2202;
const FEAT_ANALOG_HITS = 0x1b0c;
const FEAT_ONBOARD_PROFILES = 0x8100;
const FEAT_REPORT_RATE = 0x8061;

// Default feature indices on G PRO X2 Superstrike hardware table
const DEFAULT_INDICES = {
  name: 0x03,
  battery: 0x06,
  dpi: 0x09,
  hits: 0x0c,
  profiles: 0x0e,
  reportRate: 0x0d,
};

// Status file path
const STATUS_PATH = "/tmp/omarchy-superstrike-status.json";

// Enum lookups
const BATTERY_LEVEL: Record<number, string> = {
  1: "critical", 2: "low", 4: "good", 8: "full",
};
const BATTERY_STATUS: Record<number, string> = {
  0: "discharging", 1: "charging", 2: "charging_slow", 3: "full", 4: "error",
};
const LOD_LABEL: Record<number, string> = {
  0: "unsupported", 1: "low", 2: "medium", 3: "high",
};

// Report rate code <-> Hz mapping
const RATE_MAP: Record<number, number> = {
  0: 125, 1: 250, 2: 500, 3: 1000, 4: 2000, 5: 4000, 6: 8000,
};
const CODE_MAP: Record<number, number> = {
  125: 0, 250: 1, 500: 2, 1000: 3, 2000: 4, 4000: 5, 8000: 6,
};

// Default fallback DPI presets
const FALLBACK_DPI_PRESETS = [800, 1200, 1600, 2400, 3200];

// ── Types ────────────────────────────────────────────────────────────────

interface BatteryInfo {
  percentage: number;
  level: string;
  status: string;
}

interface DpiInfo {
  dpiX: number;
  defaultDpiX: number;
  dpiY: number;
  defaultDpiY: number;
  lod: string;
}

interface ButtonRecord {
  actuation: number;     // user level 1–10 (downward trigger travel)
  rapidTrigger: number;  // user level 1–5  (upward reset travel)
  haptics: number;       // user level 1–6  (click vibration strength)
}

interface HitsInfo {
  left: ButtonRecord;
  right: ButtonRecord;
}

interface MouseStatus {
  connected: boolean;
  deviceName: string;
  battery: BatteryInfo | null;
  dpi: DpiInfo | null;
  dpiMin: number;
  dpiMax: number;
  dpiPresets: number[];
  reportRate: number;
  hasHits: boolean;
  hits: HitsInfo | null;
  error: string | null;
  updatedAt: number;
}

interface FeatureIndices {
  name: number;
  battery: number;
  dpi: number;
  hits: number;
  profiles: number;
  reportRate: number;
}

// ── Transport ────────────────────────────────────────────────────────────

class HidppError extends Error {
  constructor(msg: string) {
    super(msg);
    this.name = "HidppError";
  }
}

class HidppDevice {
  private fd: number;
  readonly path: string;
  devIdx: number;

  constructor(path: string) {
    this.path = path;
    this.fd = openSync(path, constants.O_RDWR | constants.O_NONBLOCK);
    this.devIdx = 1;
  }

  close(): void {
    try { closeSync(this.fd); } catch {}
  }

  private drain(): void {
    const buf = Buffer.alloc(64);
    for (let i = 0; i < 32; i++) {
      try {
        const n = readSync(this.fd, buf, 0, 64, null);
        if (n <= 0) break;
      } catch { break; }
    }
  }

  request(featIdx: number, fn: number, params: number[] = [], tries = 3, timeoutMs = 400): Buffer {
    const pkt = Buffer.alloc(LONG_LEN);
    pkt[0] = REPORT_LONG;
    pkt[1] = this.devIdx;
    pkt[2] = featIdx;
    pkt[3] = (fn << 4) | SWID;
    for (let i = 0; i < params.length && i + 4 < LONG_LEN; i++) {
      pkt[4 + i] = params[i];
    }

    let lastErr: number | null = null;
    const resp = Buffer.alloc(64);

    for (let attempt = 0; attempt < tries; attempt++) {
      this.drain();
      try {
        writeSync(this.fd, pkt, 0, LONG_LEN, null);
      } catch (e: unknown) {
        const msg = e instanceof Error ? e.message : String(e);
        throw new HidppError(`write failed on ${this.path}: ${msg}`);
      }

      const deadline = Date.now() + timeoutMs;
      while (Date.now() < deadline) {
        let n = 0;
        try {
          n = readSync(this.fd, resp, 0, 64, null);
        } catch {
          Bun.sleepSync(4);
          continue;
        }
        if (n < 4) {
          Bun.sleepSync(4);
          continue;
        }

        if (resp[2] === 0x8f || resp[2] === 0xff) {
          lastErr = resp[5];
          break;
        }

        if (resp[1] === this.devIdx && resp[2] === featIdx && (resp[3] >> 4) === fn) {
          return Buffer.from(resp.subarray(4, n));
        }
      }
      Bun.sleepSync(15);
    }

    throw new HidppError(
      `no response feat=0x${featIdx.toString(16)} fn=${fn} (last error=${lastErr})`
    );
  }

  ping(idx: number): boolean {
    const prev = this.devIdx;
    this.devIdx = idx;
    try {
      this.request(0x00, 1, [0, 0, 0x5a], 2, 150);
      return true;
    } catch {
      this.devIdx = prev;
      return false;
    }
  }

  getFeatureIndex(featureId: number, fallback = 0): number {
    try {
      const body = this.request(0x00, 0, [featureId >> 8, featureId & 0xff], 2, 150);
      if (body && body[0] > 0) return body[0];
    } catch {}
    return fallback;
  }
}

// ── Device Discovery ─────────────────────────────────────────────────────

function hasFF00UsagePage(name: string): boolean {
  try {
    const rd = readFileSync(join("/sys/class/hidraw", name, "device/report_descriptor"));
    for (let i = 0; i < rd.length - 2; i++) {
      if (rd[i] === 0x06 && rd[i + 1] === 0x00 && rd[i + 2] === 0xff) return true;
    }
  } catch {}
  return false;
}

function sysfsVid(name: string): number {
  try {
    const uevent = readFileSync(join("/sys/class/hidraw", name, "device/uevent"), "utf-8");
    for (const line of uevent.split("\n")) {
      if (line.startsWith("HID_ID=")) {
        return parseInt(line.slice(7).split(":")[1], 16);
      }
    }
  } catch {}
  return 0;
}

function findDevice(): HidppDevice {
  const all = readdirSync("/dev")
    .filter((n) => n.startsWith("hidraw"))
    .sort();

  const preferred = all.filter((n) => sysfsVid(n) === VID && hasFF00UsagePage(n));
  const rest = all.filter((n) => !preferred.includes(n) && sysfsVid(n) === VID);
  const candidates = [...preferred, ...rest];

  for (const name of candidates) {
    const path = join("/dev", name);
    let dev: HidppDevice;
    try {
      dev = new HidppDevice(path);
    } catch { continue; }

    for (const idx of [1, 2, 3, 4, 5, 6, 0xff]) {
      if (!dev.ping(idx)) continue;
      return dev;
    }
    dev.close();
  }

  throw new Error("No Logitech HID++ device found.");
}

// ── Feature Readers & Writers ────────────────────────────────────────────

function readDeviceName(dev: HidppDevice, featIdx: number): string {
  try {
    const len = dev.request(featIdx, 0)[0];
    let nameBuf = Buffer.alloc(0);
    for (let offset = 0; offset < len; offset += 16) {
      const chunk = dev.request(featIdx, 1, [offset]);
      const take = Math.min(len - offset, 16);
      nameBuf = Buffer.concat([nameBuf, chunk.subarray(0, take)]);
    }
    const str = nameBuf.toString("utf8").replace(/\0.*$/, "").trim();
    return str !== "" ? str : "Logitech G Mouse";
  } catch {
    return "G PRO X2 SUPERSTRIKE";
  }
}

function readBattery(dev: HidppDevice, featIdx: number): BatteryInfo {
  const body = dev.request(featIdx, 1, [0, 0, 0]);
  return {
    percentage: body[0],
    level: BATTERY_LEVEL[body[1]] ?? "unknown",
    status: BATTERY_STATUS[body[2]] ?? "unknown",
  };
}

function readDpi(dev: HidppDevice, featIdx: number): DpiInfo {
  const body = dev.request(featIdx, 5, [0, 0, 0]);
  return {
    dpiX: (body[1] << 8) | body[2],
    defaultDpiX: (body[3] << 8) | body[4],
    dpiY: (body[5] << 8) | body[6],
    defaultDpiY: (body[7] << 8) | body[8],
    lod: LOD_LABEL[body[9]] ?? "unknown",
  };
}

function readDpiPresets(dev: HidppDevice, featIdx: number): number[] {
  const presets: number[] = [];
  try {
    const raw = dev.request(featIdx, 3, [0, 0, 0]);
    for (let i = 0; i < 14; i += 2) {
      const val = (raw[i] << 8) | raw[i + 1];
      if (val > 0) presets.push(val);
    }
  } catch {}
  return presets.length > 0 ? presets : FALLBACK_DPI_PRESETS;
}

function readMaxDpi(dev: HidppDevice, featIdx: number): number {
  let max = 32000;
  try {
    const p2 = dev.request(featIdx, 2, [0, 0, 2]);
    for (let i = 0; i < 14; i += 2) {
      const val = (p2[i] << 8) | p2[i + 1];
      if (val > 1000 && val <= 44000 && (val % 1000 === 0 || val === 25600)) {
        max = Math.max(max, val);
      }
    }
  } catch {}
  return max;
}

function setDpi(dev: HidppDevice, featDpi: number, featProfiles: number, targetDpi: number, maxDpi = 32000): void {
  const clamped = Math.max(100, Math.min(maxDpi, Math.round(targetDpi / 50) * 50));
  if (featProfiles > 0) {
    try { dev.request(featProfiles, 1, [2]); } catch {}
  }
  const hi = (clamped >> 8) & 0xff;
  const lo = clamped & 0xff;
  dev.request(featDpi, 6, [0, hi, lo, hi, lo, 2]);
}

function readReportRate(dev: HidppDevice, featIdx: number): number {
  try {
    const b = dev.request(featIdx, 2, [1]);
    return RATE_MAP[b[0]] ?? 1000;
  } catch {
    try {
      const b = dev.request(featIdx, 2, [0]);
      return RATE_MAP[b[0]] ?? 1000;
    } catch {
      return 1000;
    }
  }
}

function setReportRate(dev: HidppDevice, featIdx: number, rate: number): void {
  const code = CODE_MAP[rate] ?? 3;
  try { dev.request(featIdx, 3, [code, 0, 0]); } catch {}
  try { dev.request(featIdx, 3, [code, 1, 0]); } catch {}
}

function readButton(dev: HidppDevice, featIdx: number, btn: number): ButtonRecord {
  const body = dev.request(featIdx, 2, [btn]);
  return {
    actuation: Math.round(body[1] / 4),       // level 1–10
    rapidTrigger: Math.round(body[2] / 4),    // level 1–5
    haptics: Math.round(body[3] / 4) + 1,     // level 1–6
  };
}

function setButton(
  dev: HidppDevice,
  featIdx: number,
  btn: number,
  actuationLevel?: number,
  rapidTriggerLevel?: number,
  hapticsLevel?: number
): ButtonRecord {
  const current = dev.request(featIdx, 2, [btn]);
  const actByte = actuationLevel !== undefined ? actuationLevel * 4 : current[1];
  const rtByte = rapidTriggerLevel !== undefined ? rapidTriggerLevel * 4 : current[2];
  const hapByte = hapticsLevel !== undefined ? (hapticsLevel - 1) * 4 : current[3];

  const body = dev.request(featIdx, 1, [btn, actByte, rtByte, hapByte]);
  return {
    actuation: Math.round(body[1] / 4),
    rapidTrigger: Math.round(body[2] / 4),
    haptics: Math.round(body[3] / 4) + 1,
  };
}

function readHits(dev: HidppDevice, featIdx: number): HitsInfo {
  return {
    left: readButton(dev, featIdx, 0),
    right: readButton(dev, featIdx, 1),
  };
}

// ── Poll Cycle ───────────────────────────────────────────────────────────

function poll(dev: HidppDevice, features: FeatureIndices): MouseStatus {
  const status: MouseStatus = {
    connected: true,
    deviceName: "Logitech G Mouse",
    battery: null,
    dpi: null,
    dpiMin: 100,
    dpiMax: 32000,
    dpiPresets: FALLBACK_DPI_PRESETS,
    reportRate: 1000,
    hasHits: features.hits > 0,
    hits: null,
    error: null,
    updatedAt: Date.now(),
  };

  const errors: string[] = [];

  if (features.name > 0) {
    try { status.deviceName = readDeviceName(dev, features.name); }
    catch (e: unknown) { errors.push(`name: ${e instanceof Error ? e.message : e}`); }
  }
  if (features.battery > 0) {
    try { status.battery = readBattery(dev, features.battery); }
    catch (e: unknown) { errors.push(`battery: ${e instanceof Error ? e.message : e}`); }
  }
  if (features.dpi > 0) {
    try {
      status.dpi = readDpi(dev, features.dpi);
      status.dpiPresets = readDpiPresets(dev, features.dpi);
      status.dpiMax = readMaxDpi(dev, features.dpi);
    } catch (e: unknown) { errors.push(`dpi: ${e instanceof Error ? e.message : e}`); }
  }
  if (features.reportRate > 0) {
    try { status.reportRate = readReportRate(dev, features.reportRate); }
    catch (e: unknown) { errors.push(`reportRate: ${e instanceof Error ? e.message : e}`); }
  }
  if (features.hits > 0) {
    try { status.hits = readHits(dev, features.hits); }
    catch (e: unknown) { errors.push(`hits: ${e instanceof Error ? e.message : e}`); }
  }

  if (errors.length > 0) status.error = errors.join("; ");
  return status;
}

function writeStatusFile(status: MouseStatus): void {
  try {
    writeFileSync(STATUS_PATH, JSON.stringify(status) + "\n", "utf-8");
  } catch {}
}

// ── Main ─────────────────────────────────────────────────────────────────

function main(): void {
  const args = process.argv.slice(2);
  let interval = 15;
  let once = false;
  let setActuation: number | undefined;
  let setRapidTrigger: number | undefined;
  let setHaptics: number | undefined;
  let targetDpi: number | undefined;
  let targetRate: number | undefined;
  let targetLeft = false;
  let targetRight = false;

  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--interval" && args[i + 1]) {
      interval = Math.max(1, parseInt(args[i + 1], 10) || 15);
      i++;
    } else if (args[i] === "--once") {
      once = true;
    } else if (args[i] === "--set-actuation" && args[i + 1]) {
      setActuation = Math.max(1, Math.min(10, parseInt(args[i + 1], 10)));
      i++;
    } else if ((args[i] === "--set-rapid-trigger" || args[i] === "--set-rt") && args[i + 1]) {
      setRapidTrigger = Math.max(1, Math.min(5, parseInt(args[i + 1], 10)));
      i++;
    } else if (args[i] === "--set-haptics" && args[i + 1]) {
      setHaptics = Math.max(1, Math.min(6, parseInt(args[i + 1], 10)));
      i++;
    } else if (args[i] === "--set-dpi" && args[i + 1]) {
      targetDpi = parseInt(args[i + 1], 10);
      i++;
    } else if ((args[i] === "--set-report-rate" || args[i] === "--set-rate") && args[i + 1]) {
      targetRate = parseInt(args[i + 1], 10);
      i++;
    } else if (args[i] === "--left") {
      targetLeft = true;
    } else if (args[i] === "--right") {
      targetRight = true;
    }
  }

  let dev: HidppDevice;
  try {
    dev = findDevice();
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    const err: MouseStatus = {
      connected: false,
      deviceName: "Logitech G Mouse",
      battery: null,
      dpi: null,
      dpiMin: 100,
      dpiMax: 32000,
      dpiPresets: FALLBACK_DPI_PRESETS,
      reportRate: 1000,
      hasHits: false,
      hits: null,
      error: msg,
      updatedAt: Date.now()
    };
    writeStatusFile(err);
    console.log(JSON.stringify(err));
    process.exit(1);
  }

  const features: FeatureIndices = {
    name: DEFAULT_INDICES.name,
    battery: DEFAULT_INDICES.battery,
    dpi: DEFAULT_INDICES.dpi,
    hits: DEFAULT_INDICES.hits,
    profiles: DEFAULT_INDICES.profiles,
    reportRate: DEFAULT_INDICES.reportRate,
  };

  // Handle write actions
  if (setActuation !== undefined || setRapidTrigger !== undefined || setHaptics !== undefined) {
    const buttons = targetLeft ? [0] : targetRight ? [1] : [0, 1];
    for (const btn of buttons) {
      setButton(dev, features.hits, btn, setActuation, setRapidTrigger, setHaptics);
    }
  }

  if (targetDpi !== undefined && features.dpi > 0) {
    setDpi(dev, features.dpi, features.profiles, targetDpi);
  }

  if (targetRate !== undefined && features.reportRate > 0) {
    setReportRate(dev, features.reportRate, targetRate);
  }

  const emit = () => {
    try {
      const status = poll(dev, features);
      writeStatusFile(status);
      console.log(JSON.stringify(status));
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      const err: MouseStatus = {
        connected: false,
        deviceName: "Logitech G Mouse",
        battery: null,
        dpi: null,
        dpiMin: 100,
        dpiMax: 32000,
        dpiPresets: FALLBACK_DPI_PRESETS,
        reportRate: 1000,
        hasHits: false,
        hits: null,
        error: msg,
        updatedAt: Date.now()
      };
      writeStatusFile(err);
      console.log(JSON.stringify(err));
    }
  };

  emit();
  if (once || setActuation !== undefined || setRapidTrigger !== undefined || setHaptics !== undefined || targetDpi !== undefined || targetRate !== undefined) {
    dev.close();
    return;
  }

  setInterval(emit, interval * 1000);

  const cleanup = () => { dev.close(); process.exit(0); };
  process.on("SIGTERM", cleanup);
  process.on("SIGINT", cleanup);
}

main();
