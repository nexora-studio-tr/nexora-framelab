use framelab_core::gsi::Gsi;
use std::{
    io::{Read, Write},
    net::TcpStream,
    time::{Duration, Instant},
};
fn send(server: &Gsi, token: &str, path: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", server.port)).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .unwrap();
    let json=format!("{{\"auth\":{{\"token\":\"{token}\"}},\"map\":{{\"name\":\"de_test\"}},\"player\":{{\"name\":\"not retained\"}}}}");
    stream
        .write_all(
            format!(
                "POST /{path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: {}\r\n\r\n{json}",
                json.len()
            )
            .as_bytes(),
        )
        .unwrap();
    let mut result = String::new();
    let _ = stream.read_to_string(&mut result);
    result
}
#[test]
fn local_telemetry_requires_random_token_and_retains_only_map() {
    let server = Gsi::start().unwrap();
    assert!(send(&server, "wrong", &server.token).is_empty());
    assert!(server.map().is_none());
    assert!(send(&server, &server.token, &server.token).starts_with("HTTP/1.1 200"));
    let end = Instant::now() + Duration::from_secs(1);
    while server.map().is_none() && Instant::now() < end {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(server.map().as_deref(), Some("de_test"));
    server.reset();
    assert!(server.map().is_none());
}
#[test]
fn local_telemetry_refuses_other_routes() {
    let server = Gsi::start().unwrap();
    assert!(send(&server, &server.token, "not-the-token").is_empty());
    assert!(server.map().is_none());
}
