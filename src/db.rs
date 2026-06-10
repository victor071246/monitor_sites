use rusqlite::{Connection};
use std::path::PathBuf;

pub fn inicializar() -> Connection {
    let caminho = caminho_banco();
    let conn = Connection::open(&caminho)
        .unwrap_or_else(|e| {
            crate::log::erro(&format!("erro ao abrir banco de dados sqlite: {}", e));
            panic!();
        });
    criar_tabelas(&conn);
    conn
}

fn caminho_banco() -> PathBuf {
    let mut path = dirs::data_dir()
        .unwrap_or_else(|| {
            crate::log::erro("não foi possível resolver diretório de dados, provável erro de permissão para escrita: {}");
            panic!();
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
        id                       INTEGER PRIMARY KEY AUTOINCREMENT,
        dominio                  TEXT,
        ip                       TEXT,
        status                   TEXT NOT NULL DEFAULT 'desconhecido',
        checado_em               INTEGER,
        ultimo_horario_ativo     INTEGER,
        ultimo_horario_inativo   INTEGER,
        tempo_ativo              INTEGER NOT NULL DEFAULT 0,
        tempo_inativo            INTEGER NOT NULL DEFAULT 0
    );

    CREATE TABLE IF NOT EXISTS portas (
        id                       INTEGER PRIMARY KEY AUTOINCREMENT,
        host_id                  INTEGER NOT NULL,
        porta                    INTEGER NOT NULL,
        status                   TEXT NOT NULL DEFAULT 'desconhecido',
        esperado                 INTEGER NOT NULL DEFAULT 1,
        checado_em               INTEGER,
        ultimo_horario_ativo     INTEGER,
        ultimo_horario_inativo   INTEGER,
        tempo_ativo              INTEGER NOT NULL DEFAULT 0,
        tempo_inativo            INTEGER NOT NULL DEFAULT 0,
        FOREIGN KEY (host_id) REFERENCES hosts(id)
    );
    ")
    .unwrap_or_else(|e| {
        crate::log::erro(&format!("erro ao criar tabelas: {}", e));
        panic!();
    });
}

// ============================================================
// NOTAS DE ESTUDO
// ============================================================
//
// rusqlite::Connection
//   wrapper FFI sobre a API C do SQLite embutido (feature "bundled").
//   Connection::open() abre o arquivo .db — se não existir, SQLite cria.
//   a conexão é single-threaded por padrão; para múltiplas threads
//   seria necessário Arc<Mutex<Connection>> ou rusqlite::ConnectionManager.
//
// dirs::data_dir()
//   resolve o diretório de dados do usuário por plataforma:
//   Linux   → ~/.local/share
//   Windows → C:\Users\<user>\AppData\Roaming
//   macOS   → ~/Library/Application Support
//   retorna Option<PathBuf> — None se o OS não resolver o caminho.
//
// std::fs::create_dir_all()
//   cria o diretório e todos os pais necessários.
//   equivalente a `mkdir -p` no shell.
//   não falha se o diretório já existir.
//
// execute_batch()
//   executa múltiplos statements SQL separados por ponto e vírgula.
//   não retorna linhas — só confirma sucesso ou retorna erro.
//   CREATE TABLE IF NOT EXISTS garante idempotência:
//   pode chamar inicializar() quantas vezes quiser sem duplicar tabelas.
//
// FOREIGN KEY (host_id) REFERENCES hosts(id)
//   garante integridade referencial: não dá pra inserir uma porta
//   com host_id que não existe na tabela hosts.
//   no SQLite, FKs precisam ser habilitadas com PRAGMA foreign_keys = ON
//   — isso vai ser necessário quando adicionar inserção/deleção.
//
// unwrap_or_else vs ?
//   o operador ? propaga o erro pra quem chamou a função.
//   unwrap_or_else trata o erro ali mesmo — útil quando a função
//   não pode retornar Result (como aqui, que retorna Connection direto)
//   e quando você quer registrar no log antes de encerrar.