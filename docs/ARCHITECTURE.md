# Architecture

React/TypeScript and Radix run inside a local Tauri 2 WebView. The main window has an explicit command capability, a restrictive CSP and no generic shell/filesystem API. Browser mode uses a separate synthetic adapter and never falls back to fake hardware after a desktop failure.

The Rust core owns discovery, metrics, the experiment worker and SQLite. UI commands select files through native dialogs; the browser cannot supply arbitrary mutation paths. A profile ID resolves to a discovered Steam/CS2 path. A single-instance lock and operation gate serialize changes. Snapshot polling reads cached hardware; it does not repeatedly launch hardware probes during capture.

## Discovery

Fixed, encoded PowerShell queries read Windows CIM/Steam metadata. DXGI enumerates GPU memory using native Windows APIs instead of the truncated WMI AdapterRAM field. Missing providers remain warnings. Steam manifests are parsed with a bounded KeyValues parser; paths and profile files reject symlinks/reparse points. Multiple userdata profiles require an explicit choice.

## Measurement and evidence

The pinned PresentMon console binary is hashed before experiments. CSV parsing enforces byte/frame/schema/value bounds, targets CS2, rejects multiple process IDs and rejects ambiguous swapchains. A dominant stream must account for at least 90% of target rows. Presented FPS is `1000 / mean(frame_ms)`; 1% low is `1000 / mean(slowest ceil(1%) frame_ms)`. p95/p99 use interpolated quantiles. A stutter exceeds both 50 ms and twice the median.

Decisions use paired **run-level** log FPS ratios, Student-t intervals, a minimum 3% mean gain, a positive lower interval bound and tail/1% low regression vetoes. Five pairs are required; the assumptions of a small-sample t interval are not a universal quality guarantee. Selected candidates receive a separate holdout comparison. Large stalls and collector loss warnings invalidate automatic acceptance. PresentMon console warnings are not an independent ETW event-loss counter.

The current final profile is the single winning candidate, not a combination of individually accepted settings. This avoids claiming an untested combined profile. Hardware narrows candidate suitability; it is not a causal proof of a bottleneck.

## Worker

Request limits and consent gate execution. Existing CS2 sessions block mutation. A temporary GSI config targets an ephemeral loopback port and random token. Only fresh map state is retained; player payloads are ignored. The worker launches the selected replay with explicit process arguments, never through a shell, and captures only the returned owned process ID. Telemetry timeout, changed map, exited/delegated process, duration mismatch, cancellation and resource limits abort the run.

The live launch/GSI/config compatibility path is unverified in this release. A loaded map is a scene gate, not cryptographic proof of deterministic camera behavior. User interaction, replay/build differences, shader warmup and background load can still confound measurements.

## Transaction safety

Before file mutation, SQLite commits the original bytes, intended result, field patches and `prepared` state with FULL synchronous mode. A temporary file is flushed, the current fingerprint is rechecked and the replacement is renamed into place. State advances to `applied`. Final report + `committed` state are one SQLite transaction.

On interruption, prepared/applied entries are restored. Exact matches restore original bytes; otherwise a field merge preserves unrelated edits and refuses conflicting fields or changed schema. Temporary GSI files are journaled before creation and removed only when their hash still matches. External filesystem changes racing the final check/rename cannot be absolutely excluded; a trusted local host and cooperative filesystem remain assumptions.

An app close during an experiment requests cancellation and waits for the worker instead of abandoning it. Force termination can bypass destructors; next-start recovery is the durable fallback. CS2 must be closed before recovery writes.

## Data

Local SQLite retains at most 20 reports in the UI; older database records/backups are retained rather than silently discarding recovery evidence. Raw experiment CSV/log files have strict per-run limits and are removed after parsing. Imported files are read-only. Export omits runtime paths and identity information and uses create-new semantics. Backups are not encrypted and are not safe to publish.
