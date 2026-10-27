import type { Hardware, Metrics, Report, Snapshot } from './types';
const GiB = 1024 ** 3;
const hardware: Hardware = {
  os: 'Windows 11 · illustrative',
  cpu: 'Illustrative CPU',
  physicalCores: 8,
  logicalCores: 16,
  ramBytes: 16 * GiB,
  availableRamBytes: 10 * GiB,
  gpus: [
    {
      name: 'Illustrative GPU',
      dedicatedBytes: 8 * GiB,
      sharedBytes: 8 * GiB,
      driver: null,
      vendorId: 0,
      deviceId: 0,
    },
  ],
  display: '1920 × 1080 · 144 Hz',
  powerPlan: null,
  warnings: [],
  collectedAt: 0,
};
const sample = (factor: number): Metrics => ({
  frames: 2400,
  seconds: 24 * factor,
  fps: 100 / factor,
  lowOne: 88 / factor,
  p95Ms: 11.1 * factor,
  p99Ms: 11.35 * factor,
  medianMs: 10 * factor,
  stutters: 0,
  gpuBusyRatio: 0.87,
  displayedFps: null,
  sourceHash: 'synthetic-example',
  series: Array.from(
    { length: 140 },
    (_, i) => (9.7 + Math.sin(i * 0.8) * 0.7 + (i % 29 === 0 ? 3.1 : 0)) * factor,
  ),
  warnings: [],
});
export const demoReport: Report = {
  id: 'synthetic-example',
  createdAt: 0,
  synthetic: true,
  source: 'SYNTHETIC_EXAMPLE',
  status: 'example',
  hardware,
  buildId: null,
  replayHash: null,
  map: 'synthetic_scene',
  trials: [
    {
      candidate: {
        id: 'example',
        reason: 'EXAMPLE_ONLY',
        qualityCost: 'EDGE_SMOOTHING',
        patches: [{ key: 'setting.msaa_samples', before: '8', after: '4' }],
      },
      pairs: Array.from({ length: 5 }, () => ({ baseline: sample(1), candidate: sample(0.9) })),
      decision: {
        accepted: true,
        reason: 'GAIN_CONFIRMED',
        pairs: 5,
        fpsGainPct: 11.11,
        lowGainPct: 11.11,
        confidenceLowPct: 10.2,
        confidenceHighPct: 12.0,
      },
    },
  ],
  holdout: null,
  baseline: sample(1),
  transactionId: null,
  notes: ['NOT_A_REAL_CS2_BENCHMARK'],
};
export const demoSnapshot: Snapshot = {
  hardware,
  discovery: { installed: false, buildId: null, profiles: [], warnings: [] },
  collectorReady: false,
  gameRunning: false,
  job: null,
  reports: [],
  recovery: [],
  replaySelected: false,
  liveValidation: 'NOT_RUN_AT_OPERATOR_REQUEST',
};

const comparisonBaseline = { ...sample(1), sourceHash: 'synthetic-comparison-a' };
const comparisonCandidate = { ...sample(0.9), sourceHash: 'synthetic-comparison-b' };
export const demoComparisonReport: Report = {
  ...demoReport,
  id: 'synthetic-comparison',
  source: 'SYNTHETIC_COMPARISON',
  trials: [],
  holdout: null,
  map: null,
  baseline: comparisonBaseline,
  comparison: {
    schemaVersion: 1,
    baseline: comparisonBaseline,
    candidate: comparisonCandidate,
    baselineMetric: 'frametime',
    candidateMetric: 'frametime',
    comparable: true,
    checks: [
      'SAME_FRAME_METRIC',
      'DISTINCT_CAPTURES',
      'ENOUGH_FRAMES',
      'SIMILAR_DURATION',
      'NO_CAPTURE_QUALITY_WARNING',
    ].map((code) => ({ code, passed: true })),
    deltas: {
      fpsPct: 11.111111,
      lowOnePct: 11.111111,
      p95MsPct: -10,
      p99MsPct: -10,
      stuttersPerMinute: 0,
      displayedFpsPct: null,
    },
    notes: [
      'SCENE_HARDWARE_SETTINGS_UNVERIFIED',
      'SINGLE_PAIR_NO_CAUSAL_OR_CONFIDENCE_CLAIM',
      'IMPORT_CANNOT_APPLY_SETTINGS',
    ],
  },
  notes: ['NOT_A_REAL_CS2_BENCHMARK'],
};
