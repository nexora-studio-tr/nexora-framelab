import { describe, it, expect } from 'vitest';
import { active, estimateMinutes, metric, optimizeBlock, type Snapshot } from '../src/types';
import { demoSnapshot, demoReport } from '../src/demo';
const state = (): Snapshot => ({
  ...structuredClone(demoSnapshot),
  discovery: {
    installed: true,
    buildId: 'synthetic',
    profiles: [{ id: 'test', label: 'Synthetic', version: '16', supported: true, settings: [] }],
    warnings: [],
  },
  collectorReady: true,
  replaySelected: true,
});
describe('mutation gates', () => {
  it('web demo never enables optimization even with otherwise valid state', () => {
    const s = state();
    expect(optimizeBlock(s, s.discovery.profiles[0], 'balanced', true)).toBe('DESKTOP_REQUIRED');
  });
  it('a supported desktop configuration can proceed', () => {
    const s = state();
    expect(optimizeBlock(s, s.discovery.profiles[0], 'balanced', false)).toBeNull();
  });
  it('pending recovery takes precedence over a new experiment', () => {
    const s = state();
    s.recovery = [{ id: 'recovery', status: 'conflict', keys: [] }];
    expect(optimizeBlock(s, s.discovery.profiles[0], 'balanced', false)).toBe('RECOVERY_REQUIRED');
  });
  it('unknown game schemas stay read only', () => {
    const s = state();
    s.discovery.profiles[0].supported = false;
    expect(optimizeBlock(s, s.discovery.profiles[0], 'balanced', false)).toBe(
      'CONFIG_UNSUPPORTED_VERSION',
    );
  });
  it('missing replay, collector, game and hardware block changes', () => {
    for (const field of ['replay', 'collector', 'game', 'hardware']) {
      const s = state();
      if (field === 'replay') s.replaySelected = false;
      if (field === 'collector') s.collectorReady = false;
      if (field === 'game') s.discovery.installed = false;
      if (field === 'hardware') s.hardware.gpus = [];
      expect(optimizeBlock(s, s.discovery.profiles[0], 'balanced', false)).not.toBeNull();
    }
  });
  it('preserve mode cannot start a mutation experiment', () => {
    const s = state();
    expect(optimizeBlock(s, s.discovery.profiles[0], 'preserve', false)).toBe(
      'PRESERVE_IS_READ_ONLY',
    );
  });
  it('canceling work remains active until the worker finishes recovery', () => {
    expect(
      active({
        id: 'x',
        phase: 'capturing',
        completed: 1,
        total: 10,
        message: '',
        canceling: true,
      }),
    ).toBe(true);
    expect(
      active({
        id: 'x',
        phase: 'canceled',
        completed: 1,
        total: 10,
        message: '',
        canceling: false,
      }),
    ).toBe(false);
  });
});
describe('honest reports', () => {
  it('unavailable and nonfinite metrics are never displayed as zero', () => {
    expect(metric(undefined)).toBe('—');
    expect(metric(NaN)).toBe('—');
    expect(metric(Infinity)).toBe('—');
    expect(metric(0)).toBe('0.0');
  });
  it('sample evidence is synthetic and can never restore a real transaction', () => {
    expect(demoReport.synthetic).toBe(true);
    expect(demoReport.transactionId).toBeNull();
    expect(demoReport.source).toBe('SYNTHETIC_EXAMPLE');
  });
  it('duration estimate covers candidate search plus independent holdout', () => {
    expect(estimateMinutes(2, 5, 30)).toEqual([33, 83]);
    expect(estimateMinutes(100, 0, 30)).toEqual([33, 83]);
  });
});
