# Validation

This document separates implemented behavior from executed validation. Release binaries and the source package were built from commit `c88ae54e0ebdec3f90429d068328887444414a91`.

## Executed checks — 0.1.0-alpha.1

[GitHub Actions run 40222522741](https://github.com/nexora-studio-tr/nexora-framelab/actions/runs/40222522741) completed successfully on 25 October 2026:

| Check | Result |
| --- | --- |
| Rust core, Windows | 43 tests passed, including Windows canonical-path recovery |
| Rust core, Linux | 42 tests passed |
| Frontend unit gates | 10 tests passed |
| Browser checks | 8 checks passed: synthetic/demo gates, TR/EN, keyboard dialog, layout, no external requests/errors, automated WCAG A/AA rules |
| Build/style | Strict TypeScript, Vite production build, Prettier, rustfmt and Clippy passed |
| Native packages | Windows x64 executable, NSIS installer and portable ZIP built |
| Download integrity | Actions artifact digest, package SHA-256, 75-file source manifest and PresentMon pin verified |

Local Windows 11 checks passed for real CPU/RAM/DXGI GPU discovery, native CSV analysis (2,400 synthetic rows), synthetic-report labelling, executable launch, native CSV file selection, report rendering and JSON export. Missing CS2 correctly disabled experiment start. Exported analysis had no mutation transaction or runtime paths. No real gameplay data or private machine report was published.

The separate installer installation flow was not exercised; the portable executable was used for the local desktop checks. Automatic accessibility checks are not a certification.

Release files, checksums and the machine-readable validation summary are available with [v0.1.0-alpha.1](https://github.com/nexora-studio-tr/nexora-framelab/releases/tag/v0.1.0-alpha.1).

## Validation boundaries

- Frontend strict typecheck, production build and unit gates are exercised locally.
- Rust tests use synthetic captures and private temporary files for schema/allowlist bounds, process isolation, run-level statistics, recovery, conflicting edits and local telemetry authentication.
- Browser checks exercise the explicitly synthetic UI, TR/EN, keyboard dialogs, mutation gating, layout and automatic WCAG rules. This is not an accessibility certification.
- Native hardware/CSV checks and Windows packaging are separate from browser checks.
- **Live CS2, replay/GSI compatibility, real FPS gain and VAC/FACEIT behavior were not tested at the operator's explicit request. CS2 was not installed.**

No synthetic fixture or hosted-runner result is reported as a real game benchmark. Neither NVIDIA/AMD device coverage nor capture-overhead targets are claimed without a recorded real measurement.
