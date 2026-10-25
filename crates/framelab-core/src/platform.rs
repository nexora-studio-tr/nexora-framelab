use crate::model::*;
use std::{
    path::PathBuf,
    process::{Command, Output},
};

pub fn hidden(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let _ = command;
}
fn encode(script: &str) -> String {
    let bytes: Vec<_> = script.encode_utf16().flat_map(u16::to_le_bytes).collect();
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut output = String::new();
    for chunk in bytes.chunks(3) {
        let n = ((chunk[0] as u32) << 16)
            | ((chunk.get(1).copied().unwrap_or(0) as u32) << 8)
            | chunk.get(2).copied().unwrap_or(0) as u32;
        output.push(alphabet[((n >> 18) & 63) as usize] as char);
        output.push(alphabet[((n >> 12) & 63) as usize] as char);
        output.push(if chunk.len() > 1 {
            alphabet[((n >> 6) & 63) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            alphabet[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    output
}
fn ps(script: &str) -> Result<Output> {
    let script=format!("[Console]::OutputEncoding=[System.Text.UTF8Encoding]::new($false); $OutputEncoding=[Console]::OutputEncoding; $ProgressPreference='SilentlyContinue'; {script}");
    if !cfg!(windows) {
        return Err("WINDOWS_REQUIRED".into());
    }
    let root = std::env::var_os("SystemRoot").ok_or("WINDOWS_DIRECTORY_UNAVAILABLE")?;
    let path = PathBuf::from(root).join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let mut command = Command::new(path);
    command.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-EncodedCommand",
        &encode(&script),
    ]);
    hidden(&mut command);
    let child = command
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|_| "SYSTEM_QUERY_FAILED")?;
    let start = std::time::Instant::now();
    let mut child = child;
    loop {
        if child
            .try_wait()
            .map_err(|_| "SYSTEM_QUERY_FAILED")?
            .is_some()
        {
            break;
        }
        if start.elapsed().as_secs() > 20 {
            let _ = child.kill();
            let _ = child.wait();
            return Err("SYSTEM_QUERY_TIMEOUT".into());
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let output = child
        .wait_with_output()
        .map_err(|_| "SYSTEM_QUERY_FAILED")?;
    if !output.status.success() || output.stdout.len() > 256 * 1024 {
        return Err("SYSTEM_QUERY_FAILED".into());
    }
    Ok(output)
}
pub fn game_pids() -> Result<Vec<u32>> {
    let output=ps("$ErrorActionPreference='Stop'; $p=@(Get-Process -Name cs2 -ErrorAction SilentlyContinue | ForEach-Object {$_.Id}); ConvertTo-Json -InputObject $p -Compress")?;
    serde_json::from_slice(&output.stdout).map_err(|_| "PROCESS_QUERY_INVALID".into())
}
pub fn steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(output)=ps("$p=(Get-ItemProperty -LiteralPath 'HKCU:\\Software\\Valve\\Steam' -ErrorAction SilentlyContinue).SteamPath; ConvertTo-Json -InputObject $p -Compress"){
        if let Ok(Some(path))=serde_json::from_slice::<Option<String>>(&output.stdout){roots.push(PathBuf::from(path));}
    }
    if let Some(program) = std::env::var_os("ProgramFiles(x86)") {
        roots.push(PathBuf::from(program).join("Steam"));
    }
    roots.sort();
    roots.dedup();
    roots
}
pub fn hardware() -> Hardware {
    let mut hw = Hardware {
        collected_at: now_ms(),
        ..Hardware::default()
    };
    let script = r#"$ErrorActionPreference='Stop'
$o=Get-CimInstance Win32_OperatingSystem
$c=@(Get-CimInstance Win32_Processor)[0]
$m=Get-CimInstance Win32_ComputerSystem
$g=@(Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion,CurrentHorizontalResolution,CurrentVerticalResolution,CurrentRefreshRate)
$power=((& "$env:SystemRoot\System32\powercfg.exe" /getactivescheme) | Out-String).Trim()
[ordered]@{os=$o.Caption;cpu=$c.Name;physicalCores=$c.NumberOfCores;logicalCores=$c.NumberOfLogicalProcessors;ramBytes=$m.TotalPhysicalMemory;availableRamBytes=([uint64]$o.FreePhysicalMemory*1024);gpus=$g;power=$power} | ConvertTo-Json -Depth 4 -Compress
"#;
    match ps(script).and_then(|o| {
        serde_json::from_slice::<serde_json::Value>(&o.stdout)
            .map_err(|_| "SYSTEM_QUERY_INVALID".into())
    }) {
        Ok(v) => {
            let s = |key: &str| v[key].as_str().unwrap_or("").to_string();
            hw.os = s("os");
            hw.cpu = s("cpu");
            hw.physical_cores = v["physicalCores"].as_u64().unwrap_or(0) as u32;
            hw.logical_cores = v["logicalCores"].as_u64().unwrap_or(0) as u32;
            hw.ram_bytes = v["ramBytes"].as_u64().unwrap_or(0);
            hw.available_ram_bytes = v["availableRamBytes"].as_u64().unwrap_or(0);
            hw.power_plan = Some(s("power"));
            let cards = v["gpus"].as_array().cloned().unwrap_or_default();
            hw.gpus = dxgi();
            for gpu in &mut hw.gpus {
                gpu.driver = cards
                    .iter()
                    .find(|g| g["Name"].as_str() == Some(&gpu.name))
                    .and_then(|g| g["DriverVersion"].as_str().map(String::from));
            }
            if let Some(d) = cards
                .iter()
                .find(|g| g["CurrentHorizontalResolution"].as_u64().unwrap_or(0) > 0)
            {
                hw.display = format!(
                    "{} × {} · {} Hz",
                    d["CurrentHorizontalResolution"],
                    d["CurrentVerticalResolution"],
                    d["CurrentRefreshRate"]
                );
            }
        }
        Err(code) => hw.warnings.push(code),
    }
    if hw.gpus.is_empty() {
        hw.warnings.push("GPU_DISCOVERY_UNAVAILABLE".into());
    }
    hw
}
#[cfg(not(windows))]
fn dxgi() -> Vec<Gpu> {
    Vec::new()
}
#[cfg(windows)]
fn dxgi() -> Vec<Gpu> {
    use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};
    let mut out = Vec::new();
    unsafe {
        let Ok(factory): windows::core::Result<IDXGIFactory1> = CreateDXGIFactory1() else {
            return out;
        };
        for i in 0..16 {
            let Ok(adapter) = factory.EnumAdapters1(i) else {
                break;
            };
            let Ok(desc) = adapter.GetDesc1() else {
                continue;
            };
            if desc.Flags & 2 != 0 {
                continue;
            }
            let end = desc
                .Description
                .iter()
                .position(|c| *c == 0)
                .unwrap_or(desc.Description.len());
            out.push(Gpu {
                name: String::from_utf16_lossy(&desc.Description[..end]),
                dedicated_bytes: desc.DedicatedVideoMemory as u64,
                shared_bytes: desc.SharedSystemMemory as u64,
                driver: None,
                vendor_id: desc.VendorId,
                device_id: desc.DeviceId,
            });
        }
    }
    out
}
