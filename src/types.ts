export interface Gpu {
  name: string;
  dedicatedBytes: number;
  sharedBytes: number;
  driver: string | null;
  vendorId: number;
  deviceId: number;
}
export interface Hardware {
  os: string;
  cpu: string;
  physicalCores: number;
  logicalCores: number;
  ramBytes: number;
  availableRamBytes: number;
  gpus: Gpu[];
  display: string;
  powerPlan: string | null;
  warnings: string[];
  collectedAt: number;
}
export interface Setting {
  key: string;
  value: string;
}
export interface Profile {
  id: string;
  label: string;
  version: string;
  supported: boolean;
  settings: Setting[];
}
export interface Patch {
  key: string;
  before: string;
  after: string;
}
export interface Candidate {
  id: string;
  reason: string;
  qualityCost: string;
  patches: Patch[];
}
export interface Metrics {
  frames: number;
  seconds: number;
  fps: number;
  lowOne: number;
  p95Ms: number;
  p99Ms: number;
  medianMs: number;
  stutters: number;
  gpuBusyRatio: number | null;
  displayedFps: number | null;
  sourceHash: string;
  series: number[];
  warnings: string[];
}
export interface Pair {
  baseline: Metrics;
  candidate: Metrics;
}
export interface Decision {
  accepted: boolean;
  reason: string;
  pairs: number;
  fpsGainPct: number;
  lowGainPct: number;
  confidenceLowPct: number;
  confidenceHighPct: number;
}
export interface Trial {
  candidate: Candidate;
  pairs: Pair[];
  decision: Decision;
}
export interface Report {
  id: string;
  createdAt: number;
  synthetic: boolean;
  source: string;
  status: string;
  hardware: Hardware;
  buildId: string | null;
  replayHash: string | null;
  map: string | null;
  trials: Trial[];
  holdout: Trial | null;
  baseline: Metrics | null;
  transactionId: string | null;
  notes: string[];
}
export interface Job {
  id: string;
  phase: string;
  completed: number;
  total: number;
  message: string;
  canceling: boolean;
}
export interface Recovery {
  id: string;
  status: string;
  keys: string[];
}
export interface Snapshot {
  hardware: Hardware;
  discovery: {
    installed: boolean;
    buildId: string | null;
    profiles: Profile[];
    warnings: string[];
  };
  collectorReady: boolean;
  gameRunning: boolean;
  job: Job | null;
  reports: Report[];
  recovery: Recovery[];
  replaySelected: boolean;
  liveValidation: string;
}
export type Mode = 'balanced' | 'competitive' | 'preserve';
export type Lang = 'tr' | 'en';
export const active = (job: Job | null) =>
  Boolean(job && !['complete', 'failed', 'canceled'].includes(job.phase));
export function optimizeBlock(
  state: Snapshot | null,
  profile: Profile | undefined,
  mode: Mode,
  demo: boolean,
): string | null {
  if (demo) return 'DESKTOP_REQUIRED';
  if (!state) return 'SYSTEM_QUERY_FAILED';
  if (active(state.job)) return 'EXPERIMENT_RUNNING';
  if (state.recovery.length) return 'RECOVERY_REQUIRED';
  if (!state.discovery.installed) return 'CS2_NOT_INSTALLED';
  if (!state.collectorReady) return 'COLLECTOR_INTEGRITY_FAILED';
  if (!profile) return 'SELECT_PROFILE';
  if (!profile.supported) return 'CONFIG_UNSUPPORTED_VERSION';
  if (mode === 'preserve') return 'PRESERVE_IS_READ_ONLY';
  if (!state.replaySelected) return 'SELECT_REPLAY';
  if (!state.hardware.gpus.length || !state.hardware.ramBytes) return 'SYSTEM_QUERY_FAILED';
  return null;
}
export function metric(value: number | undefined | null, digits = 1): string {
  return typeof value === 'number' && Number.isFinite(value) ? value.toFixed(digits) : '—';
}
export function estimateMinutes(
  candidates: number,
  pairs: number,
  seconds: number,
): [number, number] {
  const runs = (Math.min(Math.max(candidates, 1), 2) + 1) * Math.min(Math.max(pairs, 5), 10) * 2;
  return [Math.ceil((runs * (seconds + 35)) / 60), Math.ceil((runs * (seconds + 135)) / 60)];
}
