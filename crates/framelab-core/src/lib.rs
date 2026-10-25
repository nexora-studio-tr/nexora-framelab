pub mod config;
pub mod gsi;
pub mod kv;
pub mod metrics;
pub mod model;
pub mod platform;
pub mod runner;
pub mod store;

use model::*;
use std::{
    fs::File,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

struct Runtime {
    hardware: Hardware,
    discovery: Discovery,
    job: Option<Job>,
    replay: Option<PathBuf>,
    game_running: bool,
    collector_ready: bool,
}
pub struct Engine {
    pub store: store::Store,
    collector: PathBuf,
    state: Mutex<Runtime>,
    operation: Mutex<()>,
    cancel: Arc<AtomicBool>,
    _lock: File,
}
impl Engine {
    pub fn open(dir: PathBuf, collector: PathBuf) -> Result<Arc<Self>> {
        use fs2::FileExt;
        std::fs::create_dir_all(&dir).map_err(|_| "STORAGE_ERROR")?;
        let lock = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(dir.join("instance.lock"))
            .map_err(|_| "STORAGE_ERROR")?;
        lock.try_lock_exclusive()
            .map_err(|_| "ANOTHER_INSTANCE_RUNNING")?;
        let store = store::Store::open(dir)?;
        let mut hardware = platform::hardware();
        let discovery = config::discover(&platform::steam_roots());
        let mut game_running = true;
        match platform::game_pids() {
            Ok(pids) if pids.is_empty() => {
                game_running = false;
                if let Err(error) = store.recover() {
                    hardware.warnings.push(error);
                }
            }
            Ok(_) => {}
            Err(code) => hardware.warnings.push(code),
        }
        let collector_ready = runner::collector_valid(&collector);
        Ok(Arc::new(Self {
            store,
            collector,
            state: Mutex::new(Runtime {
                hardware,
                discovery,
                job: None,
                replay: None,
                game_running,
                collector_ready,
            }),
            operation: Mutex::new(()),
            cancel: Arc::new(AtomicBool::new(false)),
            _lock: lock,
        }))
    }
    pub fn busy(&self) -> bool {
        self.state
            .lock()
            .map(|s| {
                s.job.as_ref().is_some_and(|j| {
                    !matches!(j.phase.as_str(), "complete" | "failed" | "canceled")
                })
            })
            .unwrap_or(true)
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        let s = self.state.lock().map_err(|_| "STATE_UNAVAILABLE")?;
        Ok(Snapshot {
            hardware: s.hardware.clone(),
            discovery: s.discovery.clone(),
            collector_ready: s.collector_ready,
            game_running: s.game_running,
            job: s.job.clone(),
            reports: self.store.reports()?,
            recovery: if s
                .job
                .as_ref()
                .is_some_and(|j| !matches!(j.phase.as_str(), "complete" | "failed" | "canceled"))
            {
                Vec::new()
            } else {
                self.store.pending()?
            },
            replay_selected: s.replay.is_some(),
            live_validation: "NOT_RUN_AT_OPERATOR_REQUEST".into(),
        })
    }
    pub fn refresh(&self) -> Result<Snapshot> {
        let _guard = self
            .operation
            .try_lock()
            .map_err(|_| "EXPERIMENT_RUNNING")?;
        if self.busy() {
            return Err("EXPERIMENT_RUNNING".into());
        }
        let mut hardware = platform::hardware();
        let discovery = config::discover(&platform::steam_roots());
        let pids = platform::game_pids();
        if pids.as_ref().is_ok_and(|p| p.is_empty()) {
            if let Err(code) = self.store.recover() {
                hardware.warnings.push(code);
            }
        }
        let running = pids.map(|p| !p.is_empty()).unwrap_or(true);
        let collector_ready = runner::collector_valid(&self.collector);
        {
            let mut s = self.state.lock().map_err(|_| "STATE_UNAVAILABLE")?;
            s.hardware = hardware;
            s.discovery = discovery;
            s.game_running = running;
            s.collector_ready = collector_ready;
        }
        let mut snapshot = self.snapshot()?;
        snapshot.game_running = running;
        Ok(snapshot)
    }
    pub fn select_replay(&self, path: &Path) -> Result<()> {
        if self.busy() {
            return Err("EXPERIMENT_RUNNING".into());
        }
        let replay = runner::validate_replay(path)?;
        self.state.lock().map_err(|_| "STATE_UNAVAILABLE")?.replay = Some(replay);
        Ok(())
    }
    pub fn analyze(&self, path: &Path) -> Result<Report> {
        let path = config::safe_file(path)?;
        let meta = path.metadata().map_err(|_| "CAPTURE_FILE_MISSING")?;
        if meta.len() > 64 * 1024 * 1024
            || !path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("csv"))
        {
            return Err("CSV_INVALID".into());
        }
        let raw = std::fs::read(path).map_err(|_| "CAPTURE_FILE_MISSING")?;
        let baseline = metrics::parse_csv(&raw, None)?;
        let hardware = self
            .state
            .lock()
            .map_err(|_| "STATE_UNAVAILABLE")?
            .hardware
            .clone();
        let report = Report {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: now_ms(),
            synthetic: false,
            source: "IMPORTED_CSV_UNVERIFIED_SCENE".into(),
            status: "analysis".into(),
            hardware,
            build_id: None,
            replay_hash: None,
            map: None,
            trials: Vec::new(),
            holdout: None,
            baseline: Some(baseline),
            transaction_id: None,
            notes: vec!["IMPORT_CANNOT_APPLY_SETTINGS".into()],
        };
        self.store.save_report(&report)?;
        Ok(report)
    }
    pub fn example(&self) -> Result<Report> {
        Ok(metrics::example(
            self.state
                .lock()
                .map_err(|_| "STATE_UNAVAILABLE")?
                .hardware
                .clone(),
        ))
    }
    pub fn start(self: &Arc<Self>, request: StartRequest) -> Result<String> {
        let _operation = self
            .operation
            .try_lock()
            .map_err(|_| "EXPERIMENT_RUNNING")?;
        if self.busy() {
            return Err("EXPERIMENT_RUNNING".into());
        }
        if !request.consent {
            return Err("CONSENT_REQUIRED".into());
        }
        if !(15..=120).contains(&request.duration_secs) || !(5..=10).contains(&request.pairs) {
            return Err("EXPERIMENT_LIMITS".into());
        }
        if !runner::collector_valid(&self.collector) {
            return Err("COLLECTOR_INTEGRITY_FAILED".into());
        }
        if !platform::game_pids()?.is_empty() {
            return Err("CLOSE_CS2_FIRST".into());
        }
        if !self.store.recover()?.is_empty() {
            return Err("RECOVERY_REQUIRED".into());
        }
        let (profile, discovery, hardware, replay) = {
            let state = self.state.lock().map_err(|_| "STATE_UNAVAILABLE")?;
            let profile = state
                .discovery
                .profiles
                .iter()
                .find(|p| p.id == request.profile_id)
                .cloned()
                .ok_or("SELECT_PROFILE")?;
            let replay = state.replay.clone().ok_or("SELECT_REPLAY")?;
            (
                profile,
                state.discovery.clone(),
                state.hardware.clone(),
                replay,
            )
        };
        if !profile.supported {
            return Err("CONFIG_UNSUPPORTED_VERSION".into());
        }
        if !discovery.installed {
            return Err("CS2_NOT_INSTALLED".into());
        }
        if config::candidates(&profile, &hardware, request.mode).is_empty() {
            return Err("NO_SAFE_CANDIDATES".into());
        }
        let expected_video_hash = sha256(config::read_video(&profile.path)?.as_bytes());
        let id = uuid::Uuid::new_v4().to_string();
        self.cancel.store(false, Ordering::Relaxed);
        self.state.lock().map_err(|_| "STATE_UNAVAILABLE")?.job = Some(Job {
            id: id.clone(),
            phase: "preparing".into(),
            completed: 0,
            total: 0,
            message: "PREPARING_EXPERIMENT".into(),
            canceling: false,
        });
        let engine = self.clone();
        let job_id = id.clone();
        std::thread::spawn(move || {
            let Ok(_operation) = engine.operation.lock() else {
                return;
            };
            let progress = |phase: &str, completed: usize, total: usize, message: &str| {
                if let Ok(mut state) = engine.state.lock() {
                    if let Some(job) = state.job.as_mut() {
                        job.phase = phase.into();
                        job.completed = completed;
                        job.total = total;
                        job.message = message.into();
                    }
                }
            };
            let experiment = runner::Experiment {
                store: &engine.store,
                profile,
                discovery,
                hardware: hardware.clone(),
                collector: engine.collector.clone(),
                replay,
                expected_video_hash,
                request,
                cancel: engine.cancel.clone(),
                progress: &progress,
            };
            let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                experiment.execute(job_id.clone())
            }))
            .unwrap_or_else(|_| Err("EXPERIMENT_PANICKED".into()));
            let cleanup = match platform::game_pids() {
                Ok(pids) if pids.is_empty() => engine.store.recover().map(|_| ()),
                _ => Err("CLOSE_CS2_FOR_RECOVERY".into()),
            };
            let (phase, message) = match (outcome, cleanup) {
                (Ok(_), Ok(())) => ("complete", "EXPERIMENT_FINISHED".to_string()),
                (Err(code), Ok(())) if code == "CANCELED" => ("canceled", code),
                (Err(code), _) => ("failed", code),
                (Ok(_), Err(code)) => ("failed", code),
            };
            if phase == "failed" || phase == "canceled" {
                let _ = engine.store.save_report(&Report {
                    id: job_id,
                    created_at: now_ms(),
                    synthetic: false,
                    source: "EXPERIMENT_ATTEMPT".into(),
                    status: phase.into(),
                    hardware,
                    build_id: None,
                    replay_hash: None,
                    map: None,
                    trials: Vec::new(),
                    holdout: None,
                    baseline: None,
                    transaction_id: None,
                    notes: vec![message.clone()],
                });
            }
            if let Ok(mut state) = engine.state.lock() {
                if let Some(job) = state.job.as_mut() {
                    job.phase = phase.into();
                    job.message = message;
                    job.canceling = false;
                }
                state.discovery = config::discover(&platform::steam_roots());
            }
        });
        Ok(id)
    }
    pub fn cancel(&self) -> Result<()> {
        if !self.busy() {
            return Err("NO_ACTIVE_EXPERIMENT".into());
        }
        self.cancel.store(true, Ordering::Relaxed);
        if let Some(job) = self
            .state
            .lock()
            .map_err(|_| "STATE_UNAVAILABLE")?
            .job
            .as_mut()
        {
            job.canceling = true;
        }
        Ok(())
    }
    pub fn restore(&self, report_id: &str) -> Result<()> {
        let _guard = self
            .operation
            .try_lock()
            .map_err(|_| "EXPERIMENT_RUNNING")?;
        if self.busy() {
            return Err("EXPERIMENT_RUNNING".into());
        }
        if !platform::game_pids()?.is_empty() {
            return Err("CLOSE_CS2_FIRST".into());
        }
        if let Some(id) = report_id.strip_prefix("recovery:") {
            if id.starts_with("owned:") {
                return self.store.clean_owned();
            }
            self.store.revert(id)?;
            return Ok(());
        }
        let mut report = self.store.report(report_id)?;
        let id = report.transaction_id.as_ref().ok_or("NOTHING_TO_RESTORE")?;
        self.store.revert(id)?;
        report.status = "restored".into();
        self.store.save_report(&report)
    }
    pub fn export(&self, report_id: &str, path: &Path) -> Result<()> {
        let report = self.store.report(report_id)?;
        if !path.is_absolute()
            || !path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("json"))
        {
            return Err("EXPORT_PATH_INVALID".into());
        }
        let parent = path.parent().ok_or("EXPORT_PATH_INVALID")?;
        config::safe_file(parent)?;
        if path.exists() {
            return Err("EXPORT_FILE_EXISTS".into());
        }
        let raw = serde_json::to_vec_pretty(&report).map_err(|_| "REPORT_INVALID")?;
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|_| "EXPORT_FAILED")?;
        file.write_all(&raw)
            .and_then(|_| file.sync_all())
            .map_err(|_| "EXPORT_FAILED".into())
    }
}
