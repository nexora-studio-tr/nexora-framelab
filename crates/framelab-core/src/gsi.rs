use crate::model::*;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
pub struct MapState {
    pub name: String,
    pub updated: Instant,
}
pub struct Gsi {
    pub port: u16,
    pub token: String,
    pub state: Arc<Mutex<Option<MapState>>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Gsi {
    pub fn start() -> Result<Self> {
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .map_err(|_| "TELEMETRY_PORT_FAILED")?;
        listener
            .set_nonblocking(true)
            .map_err(|_| "TELEMETRY_PORT_FAILED")?;
        let port = listener
            .local_addr()
            .map_err(|_| "TELEMETRY_PORT_FAILED")?
            .port();
        let token = uuid::Uuid::new_v4().simple().to_string();
        let state = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));
        let thread_token = token.clone();
        let thread_state = state.clone();
        let thread_stop = stop.clone();
        let thread = std::thread::spawn(move || {
            while !thread_stop.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut socket, peer)) => {
                        if peer.ip().is_loopback() {
                            if let Ok(Some(map)) = receive(&mut socket, &thread_token) {
                                if let Ok(mut value) = thread_state.lock() {
                                    *value = Some(MapState {
                                        name: map,
                                        updated: Instant::now(),
                                    });
                                }
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(20))
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            port,
            token,
            state,
            stop,
            thread: Some(thread),
        })
    }
    pub fn config(&self) -> String {
        format!("\"Nexora FrameLab local experiment\"\n{{\n\"uri\" \"http://127.0.0.1:{}/{}\"\n\"timeout\" \"1.0\"\n\"buffer\" \"0.1\"\n\"throttle\" \"0.5\"\n\"heartbeat\" \"1.0\"\n\"auth\" {{ \"token\" \"{}\" }}\n\"data\" {{ \"map\" \"1\" }}\n}}\n",self.port,self.token,self.token)
    }
    pub fn map(&self) -> Option<String> {
        self.state
            .lock()
            .ok()?
            .as_ref()
            .filter(|s| s.updated.elapsed() < Duration::from_secs(4))
            .map(|s| s.name.clone())
    }
    pub fn reset(&self) {
        if let Ok(mut state) = self.state.lock() {
            *state = None;
        }
    }
}
impl Drop for Gsi {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
fn receive(socket: &mut TcpStream, token: &str) -> Result<Option<String>> {
    socket
        .set_read_timeout(Some(Duration::from_millis(500)))
        .map_err(|_| "TELEMETRY_INVALID")?;
    socket
        .set_write_timeout(Some(Duration::from_millis(200)))
        .map_err(|_| "TELEMETRY_INVALID")?;
    let mut bytes = Vec::new();
    let header_end;
    loop {
        let mut b = [0; 2048];
        let n = socket.read(&mut b).map_err(|_| "TELEMETRY_INVALID")?;
        if n == 0 {
            return Err("TELEMETRY_INVALID".into());
        }
        bytes.extend_from_slice(&b[..n]);
        if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
            header_end = end + 4;
            break;
        }
        if bytes.len() > 8192 {
            return Err("TELEMETRY_INVALID".into());
        }
    }
    let header = std::str::from_utf8(&bytes[..header_end]).map_err(|_| "TELEMETRY_INVALID")?;
    if !header.starts_with(&format!("POST /{token} HTTP/1.1\r\n"))
        || header.to_ascii_lowercase().contains("transfer-encoding:")
    {
        return Err("TELEMETRY_INVALID".into());
    }
    let lengths: Vec<_> = header
        .lines()
        .filter_map(|line| line.split_once(':'))
        .filter(|(key, _)| key.eq_ignore_ascii_case("content-length"))
        .collect();
    if lengths.len() != 1 {
        return Err("TELEMETRY_INVALID".into());
    }
    let length = lengths[0]
        .1
        .trim()
        .parse::<usize>()
        .map_err(|_| "TELEMETRY_INVALID")?;
    if length > 128 * 1024 {
        return Err("TELEMETRY_INVALID".into());
    }
    while bytes.len() < header_end + length {
        let mut b = [0; 2048];
        let n = socket.read(&mut b).map_err(|_| "TELEMETRY_INVALID")?;
        if n == 0 {
            return Err("TELEMETRY_INVALID".into());
        }
        bytes.extend_from_slice(&b[..n]);
    }
    let payload: serde_json::Value =
        serde_json::from_slice(&bytes[header_end..header_end + length])
            .map_err(|_| "TELEMETRY_INVALID")?;
    if payload["auth"]["token"].as_str() != Some(token) {
        return Err("TELEMETRY_INVALID".into());
    }
    socket
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK")
        .map_err(|_| "TELEMETRY_INVALID")?;
    let Some(name) = payload["map"]["name"].as_str() else {
        return Ok(None);
    };
    if name.len() > 128
        || name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '/' | '-'))
    {
        return Ok(None);
    }
    Ok(Some(name.to_string()))
}
