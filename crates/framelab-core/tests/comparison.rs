use framelab_core::{comparison, metrics, model::*, Engine};
use std::fs;
use tempfile::TempDir;

fn csv(ms: f64, frames: usize) -> Vec<u8> {
    let mut raw =
        "Application,ProcessID,SwapChainAddress,FrameTime,GPUBusy,DisplayedTime\n".to_string();
    for _ in 0..frames {
        raw.push_str(&format!("cs2.exe,4242,0x1,{ms},8,NA\n"));
    }
    raw.into_bytes()
}
fn check(c: &CaptureComparison, code: &str) -> bool {
    c.checks.iter().find(|c| c.code == code).unwrap().passed
}

#[test]
fn observational_deltas_have_explicit_direction_and_no_statistical_decision() {
    let c = comparison::compare_csv(&csv(10.0, 2000), &csv(9.5, 2100)).unwrap();
    assert!(c.comparable);
    let d = c.deltas.as_ref().unwrap();
    assert!((d.fps_pct - 5.2631579).abs() < 0.0001);
    assert!((d.p99_ms_pct + 5.0).abs() < 0.0001);
    assert!(d.displayed_fps_pct.is_none());
    assert!(c
        .notes
        .iter()
        .any(|n| n == "SINGLE_PAIR_NO_CAUSAL_OR_CONFIDENCE_CLAIM"));
    let json = serde_json::to_value(c).unwrap();
    assert!(json.get("accepted").is_none());
    assert!(json.get("confidenceLowPct").is_none());
}
#[test]
fn identical_imports_and_bom_variants_do_not_count_as_two_captures() {
    let a = csv(10.0, 1500);
    let mut bom = vec![239, 187, 191];
    bom.extend_from_slice(&a);
    for b in [&a, &bom] {
        let c = comparison::compare_csv(&a, b).unwrap();
        assert!(!check(&c, "DISTINCT_CAPTURES"));
        assert!(c.deltas.is_none());
    }
}
#[test]
fn different_frame_semantics_withhold_deltas_even_when_values_match() {
    let a = csv(10.0, 1500);
    let b = String::from_utf8(csv(9.5, 1500))
        .unwrap()
        .replace("FrameTime", "CPUFrameTime");
    let c = comparison::compare_csv(&a, b.as_bytes()).unwrap();
    assert!(!check(&c, "SAME_FRAME_METRIC"));
    assert_eq!(c.candidate_metric, "cpuframetime");
    assert!(c.deltas.is_none());
}
#[test]
fn unequal_windows_are_not_reported_as_an_optimization() {
    let c = comparison::compare_csv(&csv(10.0, 2000), &csv(5.0, 2000)).unwrap();
    assert!(!check(&c, "SIMILAR_DURATION"));
    assert!(!c.comparable);
    assert!(c.deltas.is_none());
}
#[test]
fn thin_tail_samples_are_visible_but_not_compared() {
    let c = comparison::compare_csv(&csv(20.0, 700), &csv(19.0, 700)).unwrap();
    assert!(!check(&c, "ENOUGH_FRAMES"));
    assert_eq!(c.baseline.frames, 700);
    assert!(c.deltas.is_none());
}
#[test]
fn loading_stalls_and_excluded_streams_block_comparison() {
    let a = csv(10.0, 2000);
    let mut stall = String::from_utf8(csv(9.5, 2000)).unwrap();
    stall.push_str("cs2.exe,4242,0x1,600,8,NA\n");
    let mut secondary = String::from_utf8(csv(9.5, 2000)).unwrap();
    secondary.push_str(&"cs2.exe,4242,0x2,10,8,NA\n".repeat(50));
    for b in [stall, secondary] {
        let c = comparison::compare_csv(&a, b.as_bytes()).unwrap();
        assert!(!check(&c, "NO_CAPTURE_QUALITY_WARNING"));
        assert!(c.deltas.is_none());
    }
}
#[test]
fn stutter_delta_is_a_rate_not_a_count_for_different_durations() {
    let mut a = String::from_utf8(csv(10.0, 2000)).unwrap();
    let mut b = String::from_utf8(csv(9.5, 2100)).unwrap();
    a.push_str("cs2.exe,4242,0x1,100,8,NA\n");
    b.push_str("cs2.exe,4242,0x1,100,8,NA\n");
    let c = comparison::compare_csv(a.as_bytes(), b.as_bytes()).unwrap();
    let d = c.deltas.unwrap();
    assert!(
        (d.stutters_per_minute - (60.0 / c.candidate.seconds - 60.0 / c.baseline.seconds)).abs()
            < 1e-10
    );
}
#[test]
fn ambiguous_headers_and_multiple_processes_are_rejected() {
    let a = csv(10.0, 1500);
    let duplicate = String::from_utf8(a.clone())
        .unwrap()
        .replace("GPUBusy", "frametime");
    assert_eq!(
        comparison::compare_csv(&a, duplicate.as_bytes()).unwrap_err(),
        "CSV_SCHEMA_AMBIGUOUS"
    );
    let mut multiple = String::from_utf8(a.clone()).unwrap();
    multiple.push_str("cs2.exe,5555,0x1,10,8,NA\n");
    assert_eq!(
        comparison::compare_csv(&a, multiple.as_bytes()).unwrap_err(),
        "CSV_MULTIPLE_PROCESSES"
    );
}
#[test]
fn legacy_reports_without_comparison_still_load() {
    let original = metrics::example(Hardware::default());
    let mut value = serde_json::to_value(original).unwrap();
    value.as_object_mut().unwrap().remove("comparison");
    let report: Report = serde_json::from_value(value).unwrap();
    assert!(report.comparison.is_none());
    assert!(report.synthetic);
}
#[test]
fn optional_displayed_fps_requires_complete_finite_timing() {
    let a = String::from_utf8(csv(10.0, 2000))
        .unwrap()
        .replace(",NA", ",10");
    let b = String::from_utf8(csv(9.5, 2100))
        .unwrap()
        .replace(",NA", ",9.5");
    let c = comparison::compare_csv(a.as_bytes(), b.as_bytes()).unwrap();
    assert!((c.deltas.unwrap().displayed_fps_pct.unwrap() - 5.2631579).abs() < 0.0001);
    let partial = b.replacen(",9.5\n", ",NA\n", 1);
    assert!(comparison::compare_csv(a.as_bytes(), partial.as_bytes())
        .unwrap()
        .deltas
        .unwrap()
        .displayed_fps_pct
        .is_none());
    let tiny = b.replace(",9.5\n", ",1e-320\n");
    assert!(comparison::compare_csv(a.as_bytes(), tiny.as_bytes())
        .unwrap()
        .deltas
        .unwrap()
        .displayed_fps_pct
        .is_none());
}
#[test]
fn stored_comparison_exports_without_paths_or_transactions_and_leaves_inputs_untouched() {
    let temp = TempDir::new().unwrap();
    let a = temp.path().join("private-before.csv");
    let b = temp.path().join("private-after.csv");
    let bytes_a = csv(10.0, 2000);
    let bytes_b = csv(9.5, 2100);
    fs::write(&a, &bytes_a).unwrap();
    fs::write(&b, &bytes_b).unwrap();
    let engine = Engine::open(temp.path().join("data"), temp.path().join("missing.exe")).unwrap();
    let report = engine.compare(&a, &b).unwrap();
    assert_eq!(report.source, "IMPORTED_CSV_COMPARISON");
    assert!(report.transaction_id.is_none());
    assert!(report.trials.is_empty());
    assert!(report.holdout.is_none());
    assert!(report.hardware.cpu.is_empty());
    assert!(engine.restore(&report.id).is_err());
    let output = temp.path().join("comparison.json");
    engine.export(&report.id, &output).unwrap();
    let exported = fs::read_to_string(&output).unwrap();
    assert!(!exported.contains("private-before"));
    assert!(!exported.contains("private-after"));
    assert_eq!(fs::read(a).unwrap(), bytes_a);
    assert_eq!(fs::read(b).unwrap(), bytes_b);
    assert_eq!(engine.store.reports().unwrap().len(), 1);
}
#[test]
fn failed_second_import_never_saves_a_partial_report() {
    let temp = TempDir::new().unwrap();
    let a = temp.path().join("before.csv");
    fs::write(&a, csv(10.0, 1500)).unwrap();
    let engine = Engine::open(temp.path().join("data"), temp.path().join("missing.exe")).unwrap();
    assert!(engine
        .compare(&a, &temp.path().join("missing.csv"))
        .is_err());
    assert!(engine.store.reports().unwrap().is_empty());
}
