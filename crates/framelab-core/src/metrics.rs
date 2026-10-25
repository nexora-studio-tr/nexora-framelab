use crate::model::*;
use std::collections::BTreeMap;

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let position = (sorted.len() - 1) as f64 * p;
    let low = position.floor() as usize;
    let high = position.ceil() as usize;
    sorted[low] + (sorted[high] - sorted[low]) * (position - low as f64)
}
fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

pub fn summarize(frames: &[f64], gpu: &[f64], displayed: &[f64], hash: String) -> Result<Metrics> {
    if frames.len() < 120 || frames.len() > 1_000_000 {
        return Err("CAPTURE_FRAME_COUNT".into());
    }
    if frames
        .iter()
        .any(|x| !x.is_finite() || *x <= 0.0 || *x > 10_000.0)
    {
        return Err("CAPTURE_VALUE_INVALID".into());
    }
    let seconds = frames.iter().sum::<f64>() / 1000.0;
    if seconds < 10.0 {
        return Err("CAPTURE_TOO_SHORT".into());
    }
    let mut sorted = frames.to_vec();
    sorted.sort_by(f64::total_cmp);
    let median = percentile(&sorted, 0.5);
    let count = ((frames.len() as f64 * 0.01).ceil() as usize).max(1);
    let low_one = 1000.0 / mean(&sorted[sorted.len() - count..]);
    let mut warnings = Vec::new();
    if sorted.last().copied().unwrap_or_default() > 500.0 {
        warnings.push("LOADING_OR_MAJOR_STALL".into());
    }
    let gpu_busy_ratio = if gpu.len() == frames.len() {
        Some((gpu.iter().sum::<f64>() / frames.iter().sum::<f64>()).clamp(0.0, 4.0))
    } else {
        warnings.push("GPU_TIMING_UNAVAILABLE".into());
        None
    };
    let displayed_fps = if !displayed.is_empty() {
        Some(1000.0 / mean(displayed))
    } else {
        None
    };
    let step = frames.len().div_ceil(160);
    let series = frames.chunks(step).map(mean).collect();
    Ok(Metrics {
        frames: frames.len(),
        seconds,
        fps: 1000.0 / mean(frames),
        low_one,
        p95_ms: percentile(&sorted, 0.95),
        p99_ms: percentile(&sorted, 0.99),
        median_ms: median,
        stutters: frames
            .iter()
            .filter(|x| **x > 50.0 && **x > median * 2.0)
            .count(),
        gpu_busy_ratio,
        displayed_fps,
        source_hash: hash,
        series,
        warnings,
    })
}

pub fn parse_csv(raw: &[u8], expected_pid: Option<u32>) -> Result<Metrics> {
    if raw.len() > 64 * 1024 * 1024 {
        return Err("CAPTURE_TOO_LARGE".into());
    }
    let raw = raw.strip_prefix(&[239, 187, 191]).unwrap_or(raw);
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(false)
        .from_reader(raw);
    let headers = reader.headers().map_err(|_| "CSV_INVALID")?.clone();
    if headers.len() > 128 {
        return Err("CSV_INVALID".into());
    }
    let column = |names: &[&str]| {
        headers
            .iter()
            .position(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
    };
    let app = column(&["Application"]).ok_or("CSV_SCHEMA_UNSUPPORTED")?;
    let pid = column(&["ProcessID"]).ok_or("CSV_SCHEMA_UNSUPPORTED")?;
    let swap = column(&["SwapChainAddress"]).ok_or("CSV_SCHEMA_UNSUPPORTED")?;
    let frame = column(&["FrameTime", "CPUFrameTime", "MsBetweenPresents"])
        .ok_or("CSV_SCHEMA_UNSUPPORTED")?;
    let gpu = column(&["GPUBusy", "MsGPUBusy"]);
    let display = column(&["DisplayedTime"]);
    type Frame = (f64, Option<f64>, Option<f64>);
    let mut groups: BTreeMap<(u32, String), Vec<Frame>> = BTreeMap::new();
    let mut total = 0;
    for row in reader.records() {
        let row = row.map_err(|_| "CSV_INVALID")?;
        total += 1;
        if total > 1_000_000 {
            return Err("CAPTURE_TOO_LARGE".into());
        }
        if !row.get(app).unwrap_or("").eq_ignore_ascii_case("cs2.exe") {
            continue;
        }
        let process = row
            .get(pid)
            .unwrap_or("")
            .parse::<u32>()
            .map_err(|_| "CSV_PROCESS_INVALID")?;
        if expected_pid.is_some_and(|target| target != process) {
            continue;
        }
        let chain = row.get(swap).unwrap_or("");
        if chain.len() > 32 {
            return Err("CSV_SWAPCHAIN_INVALID".into());
        }
        let number = |index: usize| -> Result<f64> {
            let n = row
                .get(index)
                .unwrap_or("")
                .parse::<f64>()
                .map_err(|_| "CAPTURE_VALUE_INVALID")?;
            if !n.is_finite() || n < 0.0 || n > 10_000.0 {
                return Err("CAPTURE_VALUE_INVALID".into());
            }
            Ok(n)
        };
        let duration = number(frame)?;
        if duration == 0.0 {
            if total == 1 {
                continue;
            }
            return Err("CAPTURE_VALUE_INVALID".into());
        }
        let optional = |index: Option<usize>| -> Result<Option<f64>> {
            match index {
                Some(i) if !matches!(row.get(i), Some("NA" | "N/A" | "")) => number(i).map(Some),
                _ => Ok(None),
            }
        };
        groups.entry((process, chain.into())).or_default().push((
            duration,
            optional(gpu)?,
            optional(display)?,
        ));
        if groups.len() > 16 {
            return Err("CSV_TOO_MANY_STREAMS".into());
        }
    }
    let processes: std::collections::HashSet<_> = groups.keys().map(|(pid, _)| *pid).collect();
    if processes.len() > 1 {
        return Err("CSV_MULTIPLE_PROCESSES".into());
    }
    let total: usize = groups.values().map(Vec::len).sum();
    let (_, frames) = groups
        .into_iter()
        .max_by_key(|(_, v)| v.len())
        .ok_or("CSV_NO_CS2_FRAMES")?;
    if frames.len() * 10 < total * 9 {
        return Err("CSV_MULTIPLE_SWAPCHAINS".into());
    }
    let durations: Vec<_> = frames.iter().map(|x| x.0).collect();
    let gpu: Vec<_> = frames.iter().filter_map(|x| x.1).collect();
    let displayed: Vec<_> = frames
        .iter()
        .filter_map(|x| x.2)
        .filter(|x| *x > 0.0)
        .collect();
    let mut result = summarize(&durations, &gpu, &displayed, sha256(raw))?;
    if frames.len() != total {
        result.warnings.push("SECONDARY_SWAPCHAIN_EXCLUDED".into());
    }
    Ok(result)
}

pub fn decide(pairs: &[Pair]) -> Result<Decision> {
    if pairs.is_empty() || pairs.len() > 10 {
        return Err("PAIR_COUNT_INVALID".into());
    }
    if pairs.iter().any(|p| {
        [
            p.baseline.fps,
            p.candidate.fps,
            p.baseline.low_one,
            p.candidate.low_one,
            p.baseline.p99_ms,
            p.candidate.p99_ms,
        ]
        .iter()
        .any(|x| !x.is_finite() || *x <= 0.0)
    }) {
        return Err("METRICS_INVALID".into());
    }
    let gains: Vec<_> = pairs
        .iter()
        .map(|p| (p.candidate.fps / p.baseline.fps).ln())
        .collect();
    let low: Vec<_> = pairs
        .iter()
        .map(|p| (p.candidate.low_one / p.baseline.low_one).ln())
        .collect();
    let n = pairs.len();
    let average = mean(&gains);
    let low_average = mean(&low);
    let spread = if n > 1 {
        (gains.iter().map(|x| (x - average).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt()
    } else {
        0.0
    };
    let t = [
        0.0, 12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262,
    ][n.saturating_sub(1)];
    let error = t * spread / (n as f64).sqrt();
    let fps_gain_pct = average.exp_m1() * 100.0;
    let low_gain_pct = low_average.exp_m1() * 100.0;
    let confidence_low_pct = if n < 5 {
        -100.0
    } else {
        (average - error).exp_m1() * 100.0
    };
    let confidence_high_pct = if n < 5 {
        100.0
    } else {
        (average + error).exp_m1() * 100.0
    };
    let invalid = pairs.iter().any(|p| {
        p.baseline
            .warnings
            .iter()
            .chain(p.candidate.warnings.iter())
            .any(|w| w == "LOADING_OR_MAJOR_STALL" || w == "COLLECTOR_LOSS_WARNING")
    });
    let regression = pairs.iter().any(|p| {
        p.candidate.p99_ms > p.baseline.p99_ms * 1.05
            || p.candidate.stutters as f64 / p.candidate.seconds
                > p.baseline.stutters as f64 / p.baseline.seconds + 0.05
    });
    let reason = if n < 5 {
        "INSUFFICIENT_PAIRS"
    } else if invalid {
        "CAPTURE_QUALITY_FAILED"
    } else if regression || low_gain_pct < -2.0 {
        "TAIL_REGRESSION"
    } else if confidence_low_pct <= 0.0 || fps_gain_pct < 3.0 {
        "GAIN_NOT_PROVEN"
    } else {
        "GAIN_CONFIRMED"
    };
    Ok(Decision {
        accepted: reason == "GAIN_CONFIRMED",
        reason: reason.into(),
        pairs: n,
        fps_gain_pct,
        low_gain_pct,
        confidence_low_pct,
        confidence_high_pct,
    })
}

pub fn example(hardware: Hardware) -> Report {
    let mut pairs = Vec::new();
    for i in 0..5 {
        let baseline: Vec<_> = (0..2400)
            .map(|j| 10.0 + ((j * 17 + i * 13) % 19) as f64 * 0.025)
            .collect();
        let candidate: Vec<_> = baseline.iter().map(|x| x * 0.9).collect();
        pairs.push(Pair {
            baseline: summarize(&baseline, &[], &[], format!("synthetic-a-{i}"))
                .expect("fixed example"),
            candidate: summarize(&candidate, &[], &[], format!("synthetic-b-{i}"))
                .expect("fixed example"),
        });
    }
    let decision = decide(&pairs).expect("fixed pairs");
    let baseline = Some(pairs[0].baseline.clone());
    Report {
        id: "synthetic-example".into(),
        created_at: 0,
        synthetic: true,
        source: "SYNTHETIC_EXAMPLE".into(),
        status: "example".into(),
        hardware,
        build_id: None,
        replay_hash: None,
        map: Some("synthetic_scene".into()),
        trials: vec![Trial {
            candidate: Candidate {
                id: "example".into(),
                reason: "EXAMPLE_ONLY".into(),
                quality_cost: "EDGE_SMOOTHING".into(),
                patches: vec![Patch {
                    key: "setting.msaa_samples".into(),
                    before: "8".into(),
                    after: "4".into(),
                }],
            },
            pairs,
            decision,
        }],
        holdout: None,
        baseline,
        transaction_id: None,
        notes: vec!["NOT_A_REAL_CS2_BENCHMARK".into()],
    }
}
