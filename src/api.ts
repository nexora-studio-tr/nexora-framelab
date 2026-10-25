import { invoke, isTauri } from '@tauri-apps/api/core';
import { demoReport, demoSnapshot } from './demo';
import type { Mode, Report, Snapshot } from './types';
export const desktop = isTauri();
export const api = {
  snapshot: () =>
    desktop ? invoke<Snapshot>('snapshot') : Promise.resolve(structuredClone(demoSnapshot)),
  refresh: () =>
    desktop ? invoke<Snapshot>('refresh_system') : Promise.resolve(structuredClone(demoSnapshot)),
  example: () =>
    desktop ? invoke<Report>('load_example') : Promise.resolve(structuredClone(demoReport)),
  chooseReplay: () => invoke<boolean>('choose_replay'),
  importCapture: () => invoke<Report | null>('import_capture'),
  start: (profileId: string, mode: Mode, durationSecs: number) =>
    invoke<string>('start_experiment', {
      request: { profileId, mode, durationSecs, pairs: 5, consent: true },
    }),
  cancel: () => invoke<void>('cancel_experiment'),
  restore: (reportId: string) => invoke<void>('restore_profile', { reportId }),
  export: (reportId: string) => invoke<boolean>('export_report', { reportId }),
};
