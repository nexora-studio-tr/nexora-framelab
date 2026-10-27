# Changelog

## 0.1.0-alpha.2

- Compare two imported PresentMon captures through the native panel or `framelab-cli compare` without starting CS2 or changing settings.
- Withhold arithmetic deltas when metric semantics, duration, frame count, source distinctness or capture quality do not match. Normalize stutters per minute and keep incomplete displayed timing unavailable.
- Show bilingual quality checks, source fingerprints, normalized traces and explicit observational limits. Store/export comparison reports without paths or mutation transactions; older reports remain readable.
- Add twelve Rust comparison tests, browser coverage for compatible and blocked comparisons, mobile accessibility checks and native CLI CI checks. Derive package filenames from the current version.

Live CS2 testing remains omitted; no causal FPS gain or VAC/FACEIT compatibility is claimed.

## 0.1.0-alpha.1

Initial experimental release: native discovery, bounded PresentMon analysis, Version 16 graphics adapter, paired candidate/holdout worker, token-authenticated local scene gate, persistent recovery, TR/EN interface, synthetic demo and Windows packaging.

Live CS2 tests were explicitly omitted by the operator. Real FPS gains, current game-build compatibility and VAC/FACEIT behavior remain unverified. Power-plan mutation and persistent FPS-cap editing are deferred.
