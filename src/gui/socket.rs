use std::io::{Write, Read};
use std::os::unix::net::UnixStream;

pub fn enviar_comando(json: &str) -> Option<String> {
    let mut stream = UnixStream::connect("/tmp/monitor_sites.sock")
        .unwrap_or_else(|e| {
            crate::log::erro(&format!("erro ao conectar no socket IPC: {}", e));
            panic!();
        });

        stream.write_all(json.as_bytes()).ok()?;
        stream.write_all(b"\n").ok()?;

        let mut respostas = String::new();
        stream.read_to_string(&mut resposta).ok()?;
        Some(resposta)
}