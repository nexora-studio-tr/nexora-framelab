use crate::{kv, model::*};
use std::{
    io::Read,
    path::{Path, PathBuf},
};

pub fn read_limited(path: &Path, limit: u64, missing: &str, oversized: &str) -> Result<Vec<u8>> {
    let file = std::fs::File::open(path).map_err(|_| missing.to_string())?;
    let metadata = file.metadata().map_err(|_| missing.to_string())?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(oversized.into());
    }
    let mut bytes = Vec::new();
    file.take(limit.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|_| missing.to_string())?;
    if bytes.len() as u64 > limit {
        return Err(oversized.into());
    }
    Ok(bytes)
}

pub const SUPPORTED_VERSION: &str = "16";
pub fn safe_file(path: &Path) -> Result<PathBuf> {
    #[cfg(windows)]
    let local = matches!(path.components().next(), Some(std::path::Component::Prefix(p)) if matches!(p.kind(), std::path::Prefix::Disk(_) | std::path::Prefix::VerbatimDisk(_)));
    #[cfg(not(windows))]
    let local = !path.to_string_lossy().starts_with("\\\\");
    if !path.is_absolute() || !local {
        return Err("LOCAL_PATH_REQUIRED".into());
    }
    for ancestor in path.ancestors() {
        if std::fs::symlink_metadata(ancestor)
            .map(|m| is_link(&m))
            .unwrap_or(false)
        {
            return Err("LINK_PATH_REFUSED".into());
        }
    }
    path.canonicalize().map_err(|_| "FILE_UNAVAILABLE".into())
}
fn is_link(meta: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        meta.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        meta.file_type().is_symlink()
    }
}
pub fn read_video(path: &Path) -> Result<String> {
    let path = safe_file(path)?;
    let meta = std::fs::metadata(&path).map_err(|_| "FILE_UNAVAILABLE")?;
    if !meta.is_file() || meta.len() > 1024 * 1024 {
        return Err("CONFIG_TOO_LARGE".into());
    }
    String::from_utf8(read_limited(
        &path,
        1024 * 1024,
        "FILE_UNAVAILABLE",
        "CONFIG_TOO_LARGE",
    )?)
    .map_err(|_| "CONFIG_INVALID_UTF8".into())
}
pub fn inspect(path: &Path, label: &str) -> Result<Profile> {
    let content = read_video(path)?;
    let entries = kv::video(&content)?;
    let version = kv::text(&entries, "Version")
        .unwrap_or("unknown")
        .to_string();
    let settings = entries
        .iter()
        .filter_map(|e| match &e.value {
            kv::Value::Text(v, _) if e.key.starts_with("setting.") => Some(Setting {
                key: e.key.clone(),
                value: v.clone(),
            }),
            _ => None,
        })
        .collect();
    Ok(Profile {
        id: sha256(path.to_string_lossy().as_bytes())[..16].to_string(),
        label: label.into(),
        supported: version == SUPPORTED_VERSION,
        version,
        settings,
        path: path.to_path_buf(),
    })
}
pub fn validate_patches(content: &str, patches: &[Patch]) -> Result<()> {
    let entries = kv::video(content)?;
    if kv::text(&entries, "Version") != Some(SUPPORTED_VERSION) {
        return Err("CONFIG_UNSUPPORTED_VERSION".into());
    }
    if patches.is_empty() || patches.len() > 3 {
        return Err("PATCH_LIMIT".into());
    }
    let mut seen = std::collections::HashSet::new();
    for patch in patches {
        if !seen.insert(&patch.key) {
            return Err("PATCH_DUPLICATE".into());
        }
        let valid = |value: &str| match patch.key.as_str() {
            "setting.msaa_samples" => ["0", "2", "4", "8"].contains(&value),
            "setting.videocfg_ao_detail"
            | "setting.videocfg_particle_detail"
            | "setting.videocfg_texture_detail" => ["0", "1", "2"].contains(&value),
            _ => false,
        };
        if !valid(&patch.before) || !valid(&patch.after) {
            return Err("PATCH_NOT_ALLOWED".into());
        }
        if kv::text(&entries, &patch.key) != Some(patch.before.as_str()) {
            return Err("CONFIG_CONFLICT".into());
        }
    }
    Ok(())
}
pub fn candidates(profile: &Profile, hardware: &Hardware, mode: Mode) -> Vec<Candidate> {
    if !profile.supported
        || mode == Mode::Preserve
        || hardware.gpus.is_empty()
        || hardware.ram_bytes == 0
    {
        return Vec::new();
    }
    let value = |key: &str| {
        profile
            .settings
            .iter()
            .find(|s| s.key == key)
            .map(|s| s.value.as_str())
    };
    let mut out = Vec::new();
    if let Some(before) = value("setting.msaa_samples") {
        let after = match (before, mode) {
            ("8", Mode::Balanced) => Some("4"),
            ("4", Mode::Balanced) => Some("2"),
            ("8" | "4" | "2", Mode::Competitive) => Some("0"),
            _ => None,
        };
        if let Some(after) = after {
            out.push(Candidate {
                id: "msaa".into(),
                reason: "GPU_SAMPLE_COST".into(),
                quality_cost: "EDGE_SMOOTHING".into(),
                patches: vec![Patch {
                    key: "setting.msaa_samples".into(),
                    before: before.into(),
                    after: after.into(),
                }],
            });
        }
    }
    if out.len() < 2 {
        let (key, id, cost) = if mode == Mode::Competitive {
            ("setting.videocfg_ao_detail", "ao", "AMBIENT_SHADING")
        } else {
            (
                "setting.videocfg_particle_detail",
                "particles",
                "PARTICLE_DETAIL",
            )
        };
        if let Some(before) = value(key) {
            if let Ok(level) = before.parse::<u32>() {
                if level > 0 && level <= 2 {
                    out.push(Candidate {
                        id: id.into(),
                        reason: "SCENE_RENDER_COST".into(),
                        quality_cost: cost.into(),
                        patches: vec![Patch {
                            key: key.into(),
                            before: before.into(),
                            after: (level - 1).to_string(),
                        }],
                    });
                }
            }
        }
    }
    out
}
pub fn discover(steam_roots: &[PathBuf]) -> Discovery {
    let mut result = Discovery::default();
    let mut libraries = Vec::new();
    for steam in steam_roots {
        if !steam.is_dir() {
            continue;
        }
        result.steam_dir.get_or_insert(steam.clone());
        libraries.push(steam.clone());
        if let Ok(content) = read_video(&steam.join("steamapps/libraryfolders.vdf")) {
            if let Ok(root) = kv::parse(&content) {
                if let Some(folder) = kv::children(&root, "libraryfolders") {
                    for entry in folder {
                        if let kv::Value::Object(fields) = &entry.value {
                            if let Some(path) = kv::text(fields, "path") {
                                libraries.push(PathBuf::from(path));
                            }
                        }
                    }
                }
            }
        }
    }
    libraries.sort();
    libraries.dedup();
    for library in libraries {
        let manifest = library.join("steamapps/appmanifest_730.acf");
        let Ok(raw) = read_video(&manifest) else {
            continue;
        };
        let Ok(root) = kv::parse(&raw) else {
            result.warnings.push("MANIFEST_INVALID".into());
            continue;
        };
        let Some(app) = kv::children(&root, "AppState") else {
            continue;
        };
        if kv::text(app, "appid") != Some("730") {
            continue;
        }
        let install = kv::text(app, "installdir").unwrap_or("");
        if install.is_empty() || install.contains(['/', '\\']) || install == ".." {
            continue;
        }
        let game = library.join("steamapps/common").join(install);
        if !game.join("game/bin/win64/cs2.exe").is_file() {
            continue;
        }
        if result.game_dir.is_some() {
            result.warnings.push("MULTIPLE_CS2_INSTALLS".into());
            continue;
        }
        result.installed = true;
        result.build_id = kv::text(app, "buildid").map(String::from);
        result.game_dir = Some(game);
    }
    for steam in steam_roots {
        let Ok(users) = std::fs::read_dir(steam.join("userdata")) else {
            continue;
        };
        for entry in users.flatten().take(64) {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            let path = entry.path().join("730/local/cfg/cs2_video.txt");
            if path.is_file() {
                let label = format!("CS2 · …{}", &name[name.len().saturating_sub(4)..]);
                match inspect(&path, &label) {
                    Ok(p) => result.profiles.push(p),
                    Err(_) => result.warnings.push("CONFIG_UNREADABLE".into()),
                }
            }
        }
    }
    if result.profiles.len() > 1 {
        result.warnings.push("SELECT_PROFILE".into());
    }
    result
}
