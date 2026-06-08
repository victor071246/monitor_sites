use rusqlite::{Connection, Result};
use std::path::PathBuf;

pub fn inicializar() -> Result<Connection> {
    let caminho = caminho_banco();
    let conn = Connection::open(&caminho)?;
}