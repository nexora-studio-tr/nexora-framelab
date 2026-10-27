# Validation

This document separates implemented behavior from executed validation. Exact source commit, CI result and download integrity for the current release are recorded in its attached `RELEASE-VALIDATION.json` and checksums.

## Alpha.2 verification

Local Windows frontend verification passed: ten unit tests, twelve browser checks, strict TypeScript, production build, Prettier and rustfmt. Browser checks include both observational and blocked comparisons, source details, Turkish mobile layout and automated WCAG A/AA rules. A separate browser context simulates native IPC for the blocked case; this is not a native file-picker test.

The local standalone Rust toolchain lacks its standard library, so local compilation was not reported as successful. Windows/Linux Rust tests, Clippy, native CLI smoke checks, NSIS and portable packaging are release gates executed through GitHub Actions. Their exact-commit results must pass before alpha.2 is published; use the attached validation record for the final outcome.

Twelve new Rust comparison tests cover metric mismatch, identical/BOM inputs, unequal windows, thin tail samples, loading/secondary streams, normalized stutter rates, complete displayed timing, ambiguous headers/processes, legacy report loading, safe persistence/export and failed second imports. A comparison contains no mutation transaction or statistical acceptance decision. The source ZIP excludes generated Tauri ACL schemas, which are regenerated during builds.

Live CS2 testing remains omitted. No real FPS gain, game-build compatibility or VAC/FACEIT result is claimed. Alpha.1 native-window and hardware results below are historical evidence, not newly executed alpha.2 checks.

## Executed checks — 0.1.0-alpha.1

The alpha.1 binaries and source package used commit `c88ae54e0ebdec3f90429d068328887444414a91`.

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
