use interprocess::local_socket::{
    GenericNamespaced, Stream, ToNsName, traits::Stream as StreamTrait,
};
use std::io::{BufRead, BufReader, Write};

pub fn enviar_comando(json: &str) -> Option<String> {
    let nome = "monitor_sites.sock"
        .to_ns_name::<GenericNamespaced>()
        .ok()?;

    let mut stream = Stream::connect(nome)
        .unwrap_or_else(|e| {
            crate::log::erro(&format!("erro ao conectar no socket IPC: {}", e));
            panic!();
        });

        stream.write_all(json.as_bytes()).ok()?;
        stream.write_all(b"\n").ok()?;

        let mut reader = BufReader::new(stream);
        let mut resposta = String::new();
        reader.read_line(&mut resposta).ok()?;
        Some(resposta)
}