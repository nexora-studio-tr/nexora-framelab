use framelab_core::{metrics, model::*, platform};
fn main() {
    let args: Vec<_> = std::env::args().collect();
    let result: Result<serde_json::Value> = match args.get(1).map(String::as_str) {
        Some("diagnose") => {
            let discovery = framelab_core::config::discover(&platform::steam_roots());
            Ok(
                serde_json::json!({"hardware":platform::hardware(),"discovery":discovery,"liveCs2Test":"notRun"}),
            )
        }
        Some("analyze") => match args.get(2) {
            Some(path) => framelab_core::config::read_limited(
                std::path::Path::new(path),
                64 * 1024 * 1024,
                "CAPTURE_FILE_MISSING",
                "CAPTURE_TOO_LARGE",
            )
            .and_then(|raw| metrics::parse_csv(&raw, None))
            .and_then(|m| serde_json::to_value(m).map_err(|_| "REPORT_INVALID".into())),
            None => Err("CSV_PATH_REQUIRED".into()),
        },
        Some("compare") => match (args.get(2), args.get(3)) {
            (Some(a), Some(b)) => framelab_core::comparison::compare_files(
                std::path::Path::new(a), std::path::Path::new(b),
            ).and_then(|report| serde_json::to_value(report).map_err(|_| "REPORT_INVALID".into())),
            _ => Err("TWO_CSV_PATHS_REQUIRED".into()),
        },
        Some("example") => serde_json::to_value(metrics::example(Hardware::default()))
            .map_err(|_| "REPORT_INVALID".into()),
        _ => Err("Usage: framelab-cli diagnose | analyze <PresentMon.csv> | compare <baseline.csv> <candidate.csv> | example".into()),
    };
    match result {
        Ok(value) => println!(
            "{}",
            serde_json::to_string_pretty(&value).unwrap_or_default()
        ),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
