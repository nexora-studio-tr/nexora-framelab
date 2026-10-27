use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, String>;
pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
pub fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Gpu {
    pub name: String,
    pub dedicated_bytes: u64,
    pub shared_bytes: u64,
    pub driver: Option<String>,
    pub vendor_id: u32,
    pub device_id: u32,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hardware {
    pub os: String,
    pub cpu: String,
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub ram_bytes: u64,
    pub available_ram_bytes: u64,
    pub gpus: Vec<Gpu>,
    pub display: String,
    pub power_plan: Option<String>,
    pub warnings: Vec<String>,
    pub collected_at: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub id: String,
    pub label: String,
    pub version: String,
    pub supported: bool,
    pub settings: Vec<Setting>,
    #[serde(skip)]
    pub path: std::path::PathBuf,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub key: String,
    pub value: String,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Discovery {
    pub installed: bool,
    pub build_id: Option<String>,
    pub profiles: Vec<Profile>,
    pub warnings: Vec<String>,
    #[serde(skip)]
    pub game_dir: Option<std::path::PathBuf>,
    #[serde(skip)]
    pub steam_dir: Option<std::path::PathBuf>,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Balanced,
    Competitive,
    Preserve,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub key: String,
    pub before: String,
    pub after: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub id: String,
    pub reason: String,
    pub quality_cost: String,
    pub patches: Vec<Patch>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub frames: usize,
    pub seconds: f64,
    pub fps: f64,
    pub low_one: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub median_ms: f64,
    pub stutters: usize,
    pub gpu_busy_ratio: Option<f64>,
    pub displayed_fps: Option<f64>,
    pub source_hash: String,
    pub series: Vec<f64>,
    pub warnings: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pair {
    pub baseline: Metrics,
    pub candidate: Metrics,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComparisonCheck {
    pub code: String,
    pub passed: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObservedDeltas {
    pub fps_pct: f64,
    pub low_one_pct: f64,
    pub p95_ms_pct: f64,
    pub p99_ms_pct: f64,
    pub stutters_per_minute: f64,
    pub displayed_fps_pct: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureComparison {
    pub schema_version: u32,
    pub baseline: Metrics,
    pub candidate: Metrics,
    pub baseline_metric: String,
    pub candidate_metric: String,
    pub comparable: bool,
    pub checks: Vec<ComparisonCheck>,
    pub deltas: Option<ObservedDeltas>,
    pub notes: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Decision {
    pub accepted: bool,
    pub reason: String,
    pub pairs: usize,
    pub fps_gain_pct: f64,
    pub low_gain_pct: f64,
    pub confidence_low_pct: f64,
    pub confidence_high_pct: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trial {
    pub candidate: Candidate,
    pub pairs: Vec<Pair>,
    pub decision: Decision,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub id: String,
    pub created_at: u64,
    pub synthetic: bool,
    pub source: String,
    pub status: String,
    pub hardware: Hardware,
    pub build_id: Option<String>,
    pub replay_hash: Option<String>,
    pub map: Option<String>,
    pub trials: Vec<Trial>,
    pub holdout: Option<Trial>,
    pub baseline: Option<Metrics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison: Option<CaptureComparison>,
    pub transaction_id: Option<String>,
    pub notes: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub phase: String,
    pub completed: usize,
    pub total: usize,
    pub message: String,
    pub canceling: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recovery {
    pub id: String,
    pub status: String,
    pub keys: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub hardware: Hardware,
    pub discovery: Discovery,
    pub collector_ready: bool,
    pub game_running: bool,
    pub job: Option<Job>,
    pub reports: Vec<Report>,
    pub recovery: Vec<Recovery>,
    pub replay_selected: bool,
    pub live_validation: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartRequest {
    pub profile_id: String,
    pub mode: Mode,
    pub duration_secs: u32,
    pub pairs: u32,
    pub consent: bool,
}
