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
            Some(path) => std::fs::read(path)
                .map_err(|_| "CAPTURE_FILE_MISSING".into())
                .and_then(|raw| metrics::parse_csv(&raw, None))
                .and_then(|m| serde_json::to_value(m).map_err(|_| "REPORT_INVALID".into())),
            None => Err("CSV_PATH_REQUIRED".into()),
        },
        Some("example") => serde_json::to_value(metrics::example(Hardware::default()))
            .map_err(|_| "REPORT_INVALID".into()),
        _ => Err("Usage: framelab-cli diagnose | analyze <PresentMon.csv> | example".into()),
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
