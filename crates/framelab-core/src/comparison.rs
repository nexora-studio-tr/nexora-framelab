//! Arithmetic observations from two untrusted imports, never an experiment decision.
use crate::{config, metrics, model::*};
use std::path::Path;

pub fn read_capture(path: &Path) -> Result<Vec<u8>> {
    let path = config::safe_file(path)?;
    if !path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("csv"))
    {
        return Err("CSV_INVALID".into());
    }
    config::read_limited(
        &path,
        64 * 1024 * 1024,
        "CAPTURE_FILE_MISSING",
        "CAPTURE_TOO_LARGE",
    )
}

pub fn compare_files(baseline: &Path, candidate: &Path) -> Result<CaptureComparison> {
    compare_csv(&read_capture(baseline)?, &read_capture(candidate)?)
}

pub fn compare_csv(baseline: &[u8], candidate: &[u8]) -> Result<CaptureComparison> {
    let a = metrics::parse_capture(baseline, None)?;
    let b = metrics::parse_capture(candidate, None)?;
    let quality = |m: &Metrics| {
        !m.warnings.iter().any(|w| {
            matches!(
                w.as_str(),
                "LOADING_OR_MAJOR_STALL"
                    | "COLLECTOR_LOSS_WARNING"
                    | "SECONDARY_SWAPCHAIN_EXCLUDED"
            )
        })
    };
    let checks: Vec<ComparisonCheck> = [
        ("SAME_FRAME_METRIC", a.frame_metric == b.frame_metric),
        (
            "DISTINCT_CAPTURES",
            a.metrics.source_hash != b.metrics.source_hash,
        ),
        (
            "ENOUGH_FRAMES",
            a.metrics.frames >= 1000 && b.metrics.frames >= 1000,
        ),
        (
            "SIMILAR_DURATION",
            (a.metrics.seconds - b.metrics.seconds).abs()
                / a.metrics.seconds.max(b.metrics.seconds)
                <= 0.10 + f64::EPSILON,
        ),
        (
            "NO_CAPTURE_QUALITY_WARNING",
            quality(&a.metrics) && quality(&b.metrics),
        ),
    ]
    .into_iter()
    .map(|(code, passed)| ComparisonCheck {
        code: code.into(),
        passed,
    })
    .collect();
    let comparable = checks.iter().all(|c| c.passed);
    let pct = |before: f64, after: f64| (after / before - 1.0) * 100.0;
    let deltas = comparable.then(|| ObservedDeltas {
        fps_pct: pct(a.metrics.fps, b.metrics.fps),
        low_one_pct: pct(a.metrics.low_one, b.metrics.low_one),
        p95_ms_pct: pct(a.metrics.p95_ms, b.metrics.p95_ms),
        p99_ms_pct: pct(a.metrics.p99_ms, b.metrics.p99_ms),
        stutters_per_minute: b.metrics.stutters as f64 * 60.0 / b.metrics.seconds
            - a.metrics.stutters as f64 * 60.0 / a.metrics.seconds,
        displayed_fps_pct: if a.displayed_complete && b.displayed_complete {
            a.metrics
                .displayed_fps
                .zip(b.metrics.displayed_fps)
                .and_then(|(before, after)| {
                    let delta = pct(before, after);
                    delta.is_finite().then_some(delta)
                })
        } else {
            None
        },
    });
    Ok(CaptureComparison {
        schema_version: 1,
        baseline: a.metrics,
        candidate: b.metrics,
        baseline_metric: a.frame_metric,
        candidate_metric: b.frame_metric,
        comparable,
        checks,
        deltas,
        notes: vec![
            "SCENE_HARDWARE_SETTINGS_UNVERIFIED".into(),
            "SINGLE_PAIR_NO_CAUSAL_OR_CONFIDENCE_CLAIM".into(),
            "IMPORT_CANNOT_APPLY_SETTINGS".into(),
        ],
    })
}
