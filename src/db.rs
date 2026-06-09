use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub fn inicializar() -> Result<Connection> {
    let caminho = caminho_banco();
    let conn = Connection::open(&caminho)?;
    criar_tabelas(&coon);
    coon
}

fn caminho_banco() -> PathBuf {
    let mut path = dirs::data_dir()
        .unwrap_or_else(|e| {
            crate::log::erro(format!("não foi possível resolver diretório de dados, provável erro de permissão para escrita: ", e));
        });
        path.push("monitor_sites");
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|e| {
                crate::log::erro(&format!("erro ao criar diretório: {}", e));
                panic!();
            });
        path.push("monitor.db");
        path
}

fn criar_tabelas(conn: &Connection) {
    conn.execute_batch("
    CREATE TABLE IF NOT EXISTS hosts (
        id            INTEGER PRIMARY KEY AUTOINCREMENT,
        dominio       TEXT,
        ip            TEXT,
        status        TEXT NOT NULL DEFAULT 'desconhecido',
        checado_em    INTEGER,
        ultimo_up     INTEGER,
        ultimo_down   INTEGER,
        tempo_ativo   INTEGER NOT NULL DEFAULT 0,
        tempo_inativo INTEGER NOT NULL DEFAULT 0
    );

    C
}