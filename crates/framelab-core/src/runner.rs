use crate::{config, gsi::Gsi, metrics, model::*, platform, store::Store};
use std::{
    fs::{File, OpenOptions},
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

pub const COLLECTOR_SHA: &str = "b2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af";
pub fn collector_valid(path: &Path) -> bool {
    config::read_limited(path, 980320, "COLLECTOR_MISSING", "COLLECTOR_INVALID")
        .map(|raw| raw.len() == 980320 && sha256(&raw) == COLLECTOR_SHA)
        .unwrap_or(false)
}
pub fn validate_replay(path: &Path) -> Result<PathBuf> {
    let path = config::safe_file(path)?;
    let raw = path.to_string_lossy();
    if raw.len() > 240
        || raw.contains(['"', ';', '\n', '\r', '+'])
        || !path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("dem"))
    {
        return Err("REPLAY_PATH_INVALID".into());
    }
    let meta = path.metadata().map_err(|_| "REPLAY_UNAVAILABLE")?;
    if !meta.is_file() || meta.len() < 16 || meta.len() > 8 * 1024 * 1024 * 1024 {
        return Err("REPLAY_SIZE_INVALID".into());
    }
    let mut file = File::open(&path).map_err(|_| "REPLAY_UNAVAILABLE")?;
    let mut header = [0; 8];
    file.read_exact(&mut header)
        .map_err(|_| "REPLAY_UNAVAILABLE")?;
    if &header != b"PBDEMS2\0" {
        return Err("REPLAY_FORMAT_UNSUPPORTED".into());
    }
    Ok(path)
}
fn file_hash(path: &Path, cancel: &AtomicBool) -> Result<String> {
    use sha2::{Digest, Sha256};
    let mut file = File::open(path).map_err(|_| "REPLAY_UNAVAILABLE")?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 65536];
    loop {
        check_cancel(cancel)?;
        let n = file.read(&mut buffer).map_err(|_| "REPLAY_UNAVAILABLE")?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        Err("CANCELED".into())
    } else {
        Ok(())
    }
}
struct OwnedProcess(Child);
impl Drop for OwnedProcess {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
        }
        let _ = self.0.wait();
    }
}
struct Files(Vec<PathBuf>);
impl Drop for Files {
    fn drop(&mut self) {
        for p in &self.0 {
            let _ = std::fs::remove_file(p);
        }
    }
}
pub struct Experiment<'a> {
    pub store: &'a Store,
    pub profile: Profile,
    pub discovery: Discovery,
    pub hardware: Hardware,
    pub collector: PathBuf,
    pub replay: PathBuf,
    pub expected_video_hash: String,
    pub request: StartRequest,
    pub cancel: Arc<AtomicBool>,
    pub progress: &'a dyn Fn(&str, usize, usize, &str),
}
impl Experiment<'_> {
    pub fn execute(&self, id: String) -> Result<Report> {
        if !self.request.consent {
            return Err("CONSENT_REQUIRED".into());
        }
        if !(15..=120).contains(&self.request.duration_secs)
            || !(5..=10).contains(&self.request.pairs)
        {
            return Err("EXPERIMENT_LIMITS".into());
        }
        if !collector_valid(&self.collector) {
            return Err("COLLECTOR_INTEGRITY_FAILED".into());
        }
        if !platform::game_pids()?.is_empty() {
            return Err("CLOSE_CS2_FIRST".into());
        }
        if !self.store.pending()?.is_empty() {
            return Err("RECOVERY_REQUIRED".into());
        }
        let replay = validate_replay(&self.replay)?;
        let hash = file_hash(&replay, &self.cancel)?;
        let candidates = config::candidates(&self.profile, &self.hardware, self.request.mode);
        if candidates.is_empty() {
            return Err("NO_SAFE_CANDIDATES".into());
        }
        let game = self
            .discovery
            .game_dir
            .as_ref()
            .ok_or("CS2_NOT_INSTALLED")?;
        let telemetry = Gsi::start()?;
        let cfg = game.join("game/csgo/cfg").join(format!(
            "gamestate_integration_nexora_{}.cfg",
            uuid::Uuid::new_v4().simple()
        ));
        self.store.own_file(&cfg, telemetry.config().as_bytes())?;
        let mut done = 0;
        let total = (candidates.len() + 1) * self.request.pairs as usize * 2;
        let mut map: Option<String> = None;
        let mut trials = Vec::new();
        for candidate in candidates {
            let pairs = self.pairs(&candidate, &telemetry, &mut map, &mut done, total)?;
            let decision = metrics::decide(&pairs)?;
            trials.push(Trial {
                candidate,
                pairs,
                decision,
            });
        }
        let winner = trials
            .iter()
            .filter(|t| t.decision.accepted)
            .max_by(|a, b| {
                a.decision
                    .confidence_low_pct
                    .total_cmp(&b.decision.confidence_low_pct)
            })
            .map(|t| t.candidate.clone());
        let holdout = if let Some(winner) = winner {
            let pairs = self.pairs(&winner, &telemetry, &mut map, &mut done, total)?;
            let decision = metrics::decide(&pairs)?;
            Some(Trial {
                candidate: winner,
                pairs,
                decision,
            })
        } else {
            None
        };
        check_cancel(&self.cancel)?;
        let mut report = Report {
            id,
            created_at: now_ms(),
            synthetic: false,
            source: "OWNED_CS2_REPLAY".into(),
            status: "noGain".into(),
            hardware: self.hardware.clone(),
            build_id: self.discovery.build_id.clone(),
            replay_hash: Some(hash),
            map,
            trials,
            holdout,
            baseline: None,
            comparison: None,
            transaction_id: None,
            notes: vec![
                "REPLAY_RESULTS_DO_NOT_PREDICT_ALL_LIVE_MATCHES".into(),
                "SINGLE_CANDIDATE_PROFILE_WITH_INDEPENDENT_HOLDOUT".into(),
            ],
        };
        report.baseline = report
            .trials
            .first()
            .and_then(|t| t.pairs.first())
            .map(|p| p.baseline.clone());
        // Cleanup must succeed before a persistent profile/report is committed.
        drop(telemetry);
        self.store.clean_owned()?;
        check_cancel(&self.cancel)?;
        if let Some(trial) = report.holdout.as_ref().filter(|t| t.decision.accepted) {
            (self.progress)("applying", done, total, "APPLYING_VERIFIED_PROFILE");
            if !platform::game_pids()?.is_empty() {
                return Err("CLOSE_CS2_FIRST".into());
            }
            let tx = self
                .store
                .apply(&self.profile.path, &trial.candidate.patches)?;
            report.status = "applied".into();
            report.transaction_id = Some(tx.clone());
            if let Err(error) = self.store.commit_report(&tx, &report) {
                let _ = self.store.revert(&tx);
                return Err(error);
            }
        } else {
            self.store.save_report(&report)?;
        }
        Ok(report)
    }
    fn pairs(
        &self,
        candidate: &Candidate,
        gsi: &Gsi,
        map: &mut Option<String>,
        done: &mut usize,
        total: usize,
    ) -> Result<Vec<Pair>> {
        let mut pairs = Vec::new();
        for i in 0..self.request.pairs {
            let order = if i % 2 == 0 {
                [false, true]
            } else {
                [true, false]
            };
            let mut baseline = None;
            let mut changed = None;
            for is_candidate in order {
                check_cancel(&self.cancel)?;
                let sample = self.run_one(
                    if is_candidate { Some(candidate) } else { None },
                    gsi,
                    map,
                    *done,
                    total,
                )?;
                if is_candidate {
                    changed = Some(sample)
                } else {
                    baseline = Some(sample)
                }
                *done += 1;
                (self.progress)("comparing", *done, total, "RUN_FINISHED");
            }
            pairs.push(Pair {
                baseline: baseline.ok_or("RUN_MISSING")?,
                candidate: changed.ok_or("RUN_MISSING")?,
            });
        }
        Ok(pairs)
    }
    fn run_one(
        &self,
        candidate: Option<&Candidate>,
        gsi: &Gsi,
        map: &mut Option<String>,
        done: usize,
        total: usize,
    ) -> Result<Metrics> {
        if !platform::game_pids()?.is_empty() {
            return Err("CLOSE_CS2_FIRST".into());
        }
        let before = config::read_video(&self.profile.path)?;
        if sha256(before.as_bytes()) != self.expected_video_hash {
            return Err("CONFIG_CONFLICT".into());
        }
        let transaction = if let Some(candidate) = candidate {
            Some(self.store.apply(&self.profile.path, &candidate.patches)?)
        } else {
            None
        };
        let outcome = (|| {
            let game = self
                .discovery
                .game_dir
                .as_ref()
                .ok_or("CS2_NOT_INSTALLED")?;
            gsi.reset();
            (self.progress)("launching", done, total, "WAITING_FOR_REPLAY_TELEMETRY");
            let mut command = Command::new(game.join("game/bin/win64/cs2.exe"));
            command
                .current_dir(game.join("game"))
                .args(["-steam", "-novid", "+playdemo"])
                .arg(&self.replay)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            let child = command.spawn().map_err(|_| "GAME_START_FAILED")?;
            let mut child = OwnedProcess(child);
            let start = Instant::now();
            let scene = loop {
                check_cancel(&self.cancel)?;
                if child
                    .0
                    .try_wait()
                    .map_err(|_| "GAME_START_FAILED")?
                    .is_some()
                {
                    return Err("GAME_DELEGATED_OR_EXITED".into());
                }
                if let Some(scene) = gsi.map() {
                    break scene;
                }
                if start.elapsed().as_secs() > 120 {
                    return Err("REPLAY_TELEMETRY_TIMEOUT".into());
                }
                std::thread::sleep(Duration::from_millis(100));
            };
            if let Some(expected) = map {
                if *expected != scene {
                    return Err("REPLAY_MAP_CHANGED".into());
                }
            } else {
                *map = Some(scene.clone());
            }
            (self.progress)("warming", done, total, "WARMING_REPLAY");
            let warmup = Instant::now();
            while warmup.elapsed().as_secs() < 15 {
                check_cancel(&self.cancel)?;
                if gsi.map().as_deref() != Some(&scene) {
                    return Err("REPLAY_TELEMETRY_LOST".into());
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let current = self.store.dir.join("current");
            std::fs::create_dir_all(&current).map_err(|_| "STORAGE_ERROR")?;
            let nonce = uuid::Uuid::new_v4().simple().to_string();
            let csv = current.join(format!("{nonce}.csv"));
            let log = current.join(format!("{nonce}.log"));
            let _files = Files(vec![csv.clone(), log.clone()]);
            let output = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&log)
                .map_err(|_| "CAPTURE_START_FAILED")?;
            let mut capture = Command::new(&self.collector);
            capture
                .args([
                    "--process_id",
                    &child.0.id().to_string(),
                    "--timed",
                    &self.request.duration_secs.to_string(),
                    "--terminate_after_timed",
                    "--v2_metrics",
                    "--no_console_stats",
                    "--session_name",
                    &format!("Nexora-{nonce}"),
                    "--output_file",
                ])
                .arg(&csv)
                .stdin(Stdio::null())
                .stdout(Stdio::from(
                    output.try_clone().map_err(|_| "CAPTURE_START_FAILED")?,
                ))
                .stderr(Stdio::from(output));
            platform::hidden(&mut capture);
            let mut collector = OwnedProcess(capture.spawn().map_err(|_| "CAPTURE_START_FAILED")?);
            (self.progress)("capturing", done, total, "CAPTURING_CS2");
            let clock = Instant::now();
            loop {
                check_cancel(&self.cancel)?;
                if let Some(status) = collector.0.try_wait().map_err(|_| "COLLECTOR_FAILED")? {
                    if !status.success() {
                        return Err("COLLECTOR_PERMISSION_OR_CAPTURE_FAILED".into());
                    }
                    break;
                }
                if child.0.try_wait().map_err(|_| "GAME_EXITED")?.is_some() {
                    return Err("GAME_EXITED".into());
                }
                if gsi.map().as_deref() != Some(&scene) {
                    return Err("REPLAY_TELEMETRY_LOST".into());
                }
                if clock.elapsed().as_secs() > self.request.duration_secs as u64 + 15 {
                    return Err("CAPTURE_TIMEOUT".into());
                }
                if csv
                    .metadata()
                    .map(|m| m.len() > 64 * 1024 * 1024)
                    .unwrap_or(false)
                    || log
                        .metadata()
                        .map(|m| m.len() > 1024 * 1024)
                        .unwrap_or(false)
                {
                    return Err("CAPTURE_TOO_LARGE".into());
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            let raw = config::read_limited(
                &csv,
                64 * 1024 * 1024,
                "CAPTURE_FILE_MISSING",
                "CAPTURE_TOO_LARGE",
            )?;
            let mut metrics = metrics::parse_csv(&raw, Some(child.0.id()))?;
            if metrics.seconds < (self.request.duration_secs as f64 * 0.75)
                || metrics.seconds > self.request.duration_secs as f64 * 1.3
            {
                return Err("CAPTURE_DURATION_MISMATCH".into());
            }
            let text = std::fs::read_to_string(&log)
                .unwrap_or_default()
                .to_lowercase();
            if text.contains("lost") && text.contains("event") {
                metrics.warnings.push("COLLECTOR_LOSS_WARNING".into());
            }
            drop(collector);
            drop(child);
            Ok(metrics)
        })();
        if let Some(tx) = transaction {
            if !platform::game_pids()?.is_empty() {
                return Err("CLOSE_CS2_FOR_RECOVERY".into());
            }
            self.store.revert(&tx)?;
        }
        outcome
    }
}
