# Release gates

## Experimental alpha

Hardware discovery; read-only CSV analysis; allowlisted Version 16 config adapter; paired trials plus holdout; durable recovery; TR/EN UI; synthetic demo; source/Windows packages. The user explicitly removed live CS2 acceptance from this release, so the software is labelled experimental rather than production validated.

## Before claiming verified live tuning

1. Validate the current CS2 config version and actual field meanings on a dedicated local setup.
2. Exercise replay launch, Steam delegation, GSI availability, map freshness, warmup and shutdown with real game builds.
3. Run candidate/holdout/cancellation/crash-recovery tests against actual files while checking Steam Cloud interactions.
4. Measure the collector/app overhead and repeat on named Intel, AMD and NVIDIA devices.
5. Publish anonymized positive, null and regressive results; expand the supported schema only with evidence.

Power-plan experiments, persistent FPS-cap editing, broader sensor coverage and additional games remain deferred. No date or FPS percentage is promised.
