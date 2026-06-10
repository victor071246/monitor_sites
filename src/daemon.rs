use rusqlite::Connection;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn iniciar(conn: &Connection, intervalo: u64) {
    crate::log::info("daemon iniciado");
    loop {
        executar_ciclo(conn);
        thread::sleep(Duration::from_secs(intervalo));
    }
}

fn agora_unix() -> i64{
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn executar_ciclo(conn: &Connection) {
    let hosts = match buscar_hosts(conn){
        Ok(h) => h,
        Err(e) => {
            crate::log::erro(&format!("erro ao buscar hosts: {}", e));
            return;
        }
    };

    for (host_id, dominio, ip) in &hosts {
        let endereco = dominio.as_deref().or(ip.as_deref()).unwrap_or("");
        if endereco.is_empty(){
            crate::log::aviso(&format!("host id {} sem domínio e sem io, pulando", host_id));
        }
    }
}

fn buscar_hosts(conn: &Connection) -> rusqlite::Result<Vec<(i64, Option<String>, Option<String>)>> {
    let mut stmt = conn.prepare("SELECT id, dominio, ip FROM hosts")?;
    let resultado = stmt.query_map([], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?
    .filter_map(|r| r.ok())
    .collect();
    Ok(resultado)
}