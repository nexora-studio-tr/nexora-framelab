# Validation

This document separates implemented behavior from executed validation. Final build results will be recorded in the release evidence after the CI/native checks complete.

- Frontend strict typecheck, production build and unit gates are exercised locally.
- Rust tests use synthetic captures and private temporary files for schema/allowlist bounds, process isolation, run-level statistics, recovery, conflicting edits and local telemetry authentication.
- Browser checks exercise the explicitly synthetic UI, TR/EN, keyboard dialogs, mutation gating, layout and automatic WCAG rules. This is not an accessibility certification.
- Native hardware/CSV checks and Windows packaging are separate from browser checks.
- **Live CS2, replay/GSI compatibility, real FPS gain and VAC/FACEIT behavior were not tested at the operator's explicit request. CS2 was not installed.**

No synthetic fixture or hosted-runner result is reported as a real game benchmark. Neither NVIDIA/AMD device coverage nor capture-overhead targets are claimed without a recorded real measurement.
