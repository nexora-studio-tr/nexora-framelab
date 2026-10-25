use framelab_core::{config, kv, metrics, model::*, runner, store::Store};
use std::{fs, path::Path};
use tempfile::TempDir;

const VIDEO:&str="// retained comment\n\"video.cfg\"\n{\n\"Version\" \"16\"\n\"setting.msaa_samples\" \"8\"\n\"setting.videocfg_particle_detail\" \"2\"\n\"setting.defaultres\" \"1920\"\n\"custom\" \"keep\"\n}\n";
fn patch() -> Vec<Patch> {
    vec![Patch {
        key: "setting.msaa_samples".into(),
        before: "8".into(),
        after: "4".into(),
    }]
}
fn fixture() -> (TempDir, Store, std::path::PathBuf) {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("cs2_video.txt");
    fs::write(&path, VIDEO).unwrap();
    let store = Store::open(temp.path().join("data")).unwrap();
    (temp, store, path)
}
fn sample(ms: f64) -> Metrics {
    metrics::summarize(&vec![ms; 2400], &[], &[], "synthetic".into()).unwrap()
}
fn csv(pid: u32, chain: &str, ms: &str, count: usize) -> Vec<u8> {
    let mut text =
        "Application,ProcessID,SwapChainAddress,CPUStartTime,FrameTime,GPUBusy,DisplayedTime\n"
            .to_string();
    for i in 0..count {
        text.push_str(&format!("cs2.exe,{pid},{chain},{i},{ms},8,NA\n"));
    }
    text.into_bytes()
}

#[test]
fn lossless_patch_keeps_comments_and_unrelated_fields() {
    let changed = kv::replace(VIDEO, &patch(), false).unwrap();
    assert!(changed.contains("// retained comment"));
    assert!(changed.contains("\"custom\" \"keep\""));
    assert!(changed.contains("\"setting.defaultres\" \"1920\""));
    assert_eq!(kv::replace(&changed, &patch(), true).unwrap(), VIDEO);
}
#[test]
fn parser_rejects_duplicate_keys_case_insensitively() {
    assert!(kv::parse("\"a\" \"1\" \"A\" \"2\"").is_err());
}
#[test]
fn parser_rejects_unclosed_strings_and_objects() {
    assert!(kv::parse("\"a").is_err());
    assert!(kv::parse("a { b c").is_err());
}
#[test]
fn parser_rejects_external_include() {
    assert!(kv::parse("#base something").is_err());
}
#[test]
fn parser_bounds_nesting_and_size() {
    let deep = format!("{}{}", "x {".repeat(20), "}".repeat(20));
    assert!(kv::parse(&deep).is_err());
    assert!(kv::parse(&"x".repeat(1024 * 1024 + 1)).is_err());
}
#[test]
fn parser_preserves_windows_paths_and_escaped_quotes() {
    let root = kv::parse("path \"C:\\\\Steam\\\\games\" note \"a\\\"b\"").unwrap();
    assert_eq!(kv::text(&root, "path"), Some("C:\\Steam\\games"));
    assert_eq!(kv::text(&root, "note"), Some("a\"b"));
}
#[test]
fn parser_accepts_bom_without_losing_it_on_patch() {
    let text = format!("\u{feff}{VIDEO}");
    assert!(kv::replace(&text, &patch(), false)
        .unwrap()
        .starts_with('\u{feff}'));
}
#[test]
fn schema_unknown_is_read_only() {
    let (_temp, _store, path) = fixture();
    fs::write(&path, VIDEO.replace("\"16\"", "\"99\"")).unwrap();
    let profile = config::inspect(&path, "Test").unwrap();
    assert!(!profile.supported);
    assert!(config::validate_patches(&fs::read_to_string(path).unwrap(), &patch()).is_err());
}
#[test]
fn patch_rejects_non_allowlisted_keys() {
    let mut p = patch();
    p[0].key = "setting.defaultres".into();
    p[0].before = "1920".into();
    p[0].after = "640".into();
    assert!(config::validate_patches(VIDEO, &p).is_err());
}
#[test]
fn patch_rejects_injection_and_invalid_values() {
    let mut p = patch();
    p[0].after = "4\";quit".into();
    assert!(config::validate_patches(VIDEO, &p).is_err());
    p[0].after = "3".into();
    assert!(config::validate_patches(VIDEO, &p).is_err());
}
#[test]
fn profile_choice_is_hardware_and_mode_bounded() {
    let (_temp, _store, path) = fixture();
    let profile = config::inspect(&path, "Test").unwrap();
    let mut hw = Hardware::default();
    assert!(config::candidates(&profile, &hw, Mode::Balanced).is_empty());
    hw.ram_bytes = 8 * 1024 * 1024 * 1024;
    hw.gpus.push(Gpu {
        name: "Synthetic GPU".into(),
        ..Gpu::default()
    });
    assert_eq!(
        config::candidates(&profile, &hw, Mode::Balanced)[0].patches[0].after,
        "4"
    );
    assert_eq!(
        config::candidates(&profile, &hw, Mode::Competitive)[0].patches[0].after,
        "0"
    );
    assert!(config::candidates(&profile, &hw, Mode::Preserve).is_empty());
}
#[test]
fn apply_writes_journal_before_mutation_and_restores() {
    let (_temp, store, path) = fixture();
    let id = store.apply(&path, &patch()).unwrap();
    assert_eq!(store.pending().unwrap()[0].id, id);
    assert!(fs::read_to_string(&path)
        .unwrap()
        .contains("\"setting.msaa_samples\" \"4\""));
    store.revert(&id).unwrap();
    assert_eq!(fs::read_to_string(path).unwrap(), VIDEO);
    assert!(store.pending().unwrap().is_empty());
}
#[test]
fn fresh_store_recovers_interrupted_change() {
    let (temp, store, path) = fixture();
    store.apply(&path, &patch()).unwrap();
    drop(store);
    let reopened = Store::open(temp.path().join("data")).unwrap();
    assert!(reopened.recover().unwrap().is_empty());
    assert_eq!(fs::read_to_string(path).unwrap(), VIDEO);
}
#[test]
fn rollback_merges_unrelated_user_changes() {
    let (_temp, store, path) = fixture();
    let id = store.apply(&path, &patch()).unwrap();
    let changed = fs::read_to_string(&path)
        .unwrap()
        .replace("\"keep\"", "\"user edit\"");
    fs::write(&path, changed).unwrap();
    store.revert(&id).unwrap();
    let final_text = fs::read_to_string(path).unwrap();
    assert!(final_text.contains("\"user edit\""));
    assert!(final_text.contains("\"setting.msaa_samples\" \"8\""));
}
#[test]
fn rollback_refuses_conflicting_user_field() {
    let (_temp, store, path) = fixture();
    let id = store.apply(&path, &patch()).unwrap();
    let changed = fs::read_to_string(&path).unwrap().replace(
        "\"setting.msaa_samples\" \"4\"",
        "\"setting.msaa_samples\" \"2\"",
    );
    fs::write(&path, &changed).unwrap();
    assert!(store.revert(&id).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), changed);
    assert_eq!(store.pending().unwrap()[0].status, "conflict");
}
#[test]
fn second_overlapping_mutation_is_refused() {
    let (_temp, store, path) = fixture();
    store.apply(&path, &patch()).unwrap();
    let p = vec![Patch {
        key: "setting.msaa_samples".into(),
        before: "4".into(),
        after: "2".into(),
    }];
    assert!(store.apply(&path, &p).is_err());
}
#[test]
fn committed_change_survives_restart_but_can_be_restored() {
    let (temp, store, path) = fixture();
    let id = store.apply(&path, &patch()).unwrap();
    store.commit(&id).unwrap();
    drop(store);
    let reopened = Store::open(temp.path().join("data")).unwrap();
    assert!(reopened.recover().unwrap().is_empty());
    assert!(fs::read_to_string(&path).unwrap().contains("\"4\""));
    reopened.revert(&id).unwrap();
    assert_eq!(fs::read_to_string(path).unwrap(), VIDEO);
}
#[test]
fn owned_telemetry_file_cleanup_is_hash_checked() {
    let (temp, store, _path) = fixture();
    let path = temp.path().join("owned.cfg");
    store.own_file(&path, b"owned").unwrap();
    assert!(store.pending().unwrap().iter().any(|r| r.status == "owned"));
    fs::write(&path, b"user changed").unwrap();
    assert!(store.clean_owned().is_err());
    assert_eq!(fs::read(&path).unwrap(), b"user changed");
    fs::write(&path, b"owned").unwrap();
    store.clean_owned().unwrap();
    assert!(!path.exists());
}
#[test]
fn owned_file_never_overwrites_existing_file() {
    let (_temp, store, path) = fixture();
    assert!(store.own_file(&path, b"bad").is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), VIDEO);
}
#[test]
fn restore_is_idempotent() {
    let (_temp, store, path) = fixture();
    let id = store.apply(&path, &patch()).unwrap();
    store.revert(&id).unwrap();
    store.revert(&id).unwrap();
    assert_eq!(fs::read_to_string(path).unwrap(), VIDEO);
}
#[test]
fn unknown_schema_and_stale_values_never_write() {
    let (_temp, store, path) = fixture();
    let original = VIDEO.replace("\"16\"", "\"17\"");
    fs::write(&path, &original).unwrap();
    assert!(store.apply(&path, &patch()).is_err());
    assert_eq!(fs::read_to_string(path).unwrap(), original);
}
#[test]
fn csv_metrics_are_not_fps_averages() {
    let m = metrics::summarize(
        &[vec![10.0; 1200], vec![20.0; 1200]].concat(),
        &[],
        &[],
        "test".into(),
    )
    .unwrap();
    assert!((m.fps - 66.6666667).abs() < 0.0001);
    assert_eq!(m.low_one, 50.0);
    assert_eq!(m.p99_ms, 20.0);
    assert!(m.series.len() <= 160);
}
#[test]
fn invalid_and_non_finite_frame_times_are_rejected() {
    for value in [f64::NAN, f64::INFINITY, 0.0, -1.0, 10001.0] {
        let mut data = vec![10.0; 2400];
        data[20] = value;
        assert!(metrics::summarize(&data, &[], &[], "test".into()).is_err());
    }
}
#[test]
fn short_capture_is_not_an_optimization_result() {
    assert!(metrics::summarize(&vec![1.0; 200], &[], &[], "test".into()).is_err());
    assert!(metrics::summarize(&[20.0; 119], &[], &[], "test".into()).is_err());
}
#[test]
fn csv_selects_owned_pid_without_mixing_streams() {
    let raw = csv(42, "0x1", "10", 2400);
    assert_eq!(metrics::parse_csv(&raw, Some(42)).unwrap().frames, 2400);
    assert!(metrics::parse_csv(&raw, Some(43)).is_err());
}
#[test]
fn csv_rejects_ambiguous_pids() {
    let mut raw = csv(42, "0x1", "10", 2400);
    raw.extend_from_slice(b"cs2.exe,43,0x1,1,10,8,NA\n");
    assert!(metrics::parse_csv(&raw, None).is_err());
}
#[test]
fn csv_rejects_multiple_active_swapchains() {
    let mut raw = csv(42, "0x1", "10", 2400);
    for _ in 0..1000 {
        raw.extend_from_slice(b"cs2.exe,42,0x2,1,10,8,NA\n");
    }
    assert!(metrics::parse_csv(&raw, None).is_err());
}
#[test]
fn csv_rejects_wrong_game_and_schema() {
    assert!(metrics::parse_csv(&csv(42, "x", "NaN", 2400), None).is_err());
    assert!(metrics::parse_csv(&b"a,b\n1,2\n"[..], None).is_err());
    let raw = String::from_utf8(csv(42, "x", "10", 2400))
        .unwrap()
        .replace("cs2.exe", "other.exe");
    assert!(metrics::parse_csv(raw.as_bytes(), None).is_err());
}
#[test]
fn fewer_than_five_pairs_cannot_apply() {
    let pairs = vec![
        Pair {
            baseline: sample(10.0),
            candidate: sample(8.0)
        };
        4
    ];
    assert!(!metrics::decide(&pairs).unwrap().accepted);
}
#[test]
fn noisy_small_gain_is_not_confirmed() {
    let pairs: (Vec<Pair>) = (0..5)
        .map(|i| Pair {
            baseline: sample(10.0),
            candidate: sample([9.0, 11.0, 9.8, 10.2, 9.9][i]),
        })
        .collect();
    assert!(!metrics::decide(&pairs).unwrap().accepted);
}
#[test]
fn consistent_gain_can_be_confirmed() {
    let pairs = vec![
        Pair {
            baseline: sample(10.0),
            candidate: sample(9.0)
        };
        5
    ];
    let decision = metrics::decide(&pairs).unwrap();
    assert!(decision.accepted);
    assert!(decision.confidence_low_pct > 10.0);
}
#[test]
fn low_fps_and_tail_regression_veto_mean_gain() {
    let mut candidate = sample(8.0);
    candidate.low_one = 20.0;
    candidate.p99_ms = 60.0;
    let pairs = vec![
        Pair {
            baseline: sample(10.0),
            candidate
        };
        5
    ];
    assert_eq!(metrics::decide(&pairs).unwrap().reason, "TAIL_REGRESSION");
}
#[test]
fn collector_loss_warning_vetoes_gain() {
    let mut candidate = sample(8.0);
    candidate.warnings.push("COLLECTOR_LOSS_WARNING".into());
    assert!(
        !metrics::decide(&vec![
            Pair {
                baseline: sample(10.0),
                candidate
            };
            5
        ])
        .unwrap()
        .accepted
    );
}
#[test]
fn example_is_always_labelled_synthetic() {
    let example = metrics::example(Hardware::default());
    assert!(example.synthetic);
    assert!(example.transaction_id.is_none());
    assert_eq!(example.source, "SYNTHETIC_EXAMPLE");
}
#[test]
fn replay_rejects_wrong_header_and_console_metacharacters() {
    let temp = TempDir::new().unwrap();
    let wrong = temp.path().join("wrong.dem");
    fs::write(&wrong, b"this is not a demo at all").unwrap();
    assert!(runner::validate_replay(&wrong).is_err());
    let valid = temp.path().join("local.dem");
    fs::write(&valid, b"PBDEMS2\0abcdefgh").unwrap();
    assert!(runner::validate_replay(&valid).is_ok());
    let injected = temp.path().join("x;quit.dem");
    fs::write(&injected, b"PBDEMS2\0abcdefgh").unwrap();
    assert!(runner::validate_replay(&injected).is_err());
}
#[test]
fn collector_is_hash_pinned() {
    let temp = TempDir::new().unwrap();
    let path = temp.path().join("PresentMon.exe");
    fs::write(&path, b"not official").unwrap();
    assert!(!runner::collector_valid(&path));
}
#[test]
fn steam_library_discovery_is_app730_scoped() {
    let temp = TempDir::new().unwrap();
    let steam = temp.path().join("Steam");
    let lib = temp.path().join("Library");
    fs::create_dir_all(steam.join("steamapps")).unwrap();
    fs::create_dir_all(lib.join("steamapps/common/Counter-Strike Global Offensive/game/bin/win64"))
        .unwrap();
    fs::write(
        lib.join("steamapps/common/Counter-Strike Global Offensive/game/bin/win64/cs2.exe"),
        b"fixture, never executed",
    )
    .unwrap();
    fs::write(
        lib.join("steamapps/appmanifest_730.acf"),
        "AppState { appid 730 installdir \"Counter-Strike Global Offensive\" buildid 12345 }",
    )
    .unwrap();
    let escaped = lib.to_string_lossy().replace('\\', "\\\\");
    fs::write(
        steam.join("steamapps/libraryfolders.vdf"),
        format!("libraryfolders {{ 0 {{ path \"{escaped}\" }} }}"),
    )
    .unwrap();
    let found = config::discover(&[steam]);
    assert!(found.installed);
    assert_eq!(found.build_id.as_deref(), Some("12345"));
}
#[test]
fn steam_manifest_cannot_escape_install_root() {
    let temp = TempDir::new().unwrap();
    fs::create_dir_all(temp.path().join("steamapps")).unwrap();
    fs::write(
        temp.path().join("steamapps/appmanifest_730.acf"),
        "AppState { appid 730 installdir ../escape }",
    )
    .unwrap();
    assert!(!config::discover(&[temp.path().into()]).installed);
}
#[test]
fn relative_mutation_paths_are_rejected() {
    assert!(config::safe_file(Path::new("relative/cs2_video.txt")).is_err());
}
