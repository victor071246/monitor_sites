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

pub fn inserir_host(conn: &Connection, dominio: Option<&str>, ip: Option<&str>) -> i64 {
    conn.execute(
        "INSERT INTO hosts (dominio, ip) VALUES (?1, ?2)",
        rusqlite::params![dominio, ip],
    )
    .unwrap_or_else(|e| {
        crate::log::erro(&format!("erro ao inserir host: {}", e));
        panic!();
    });
    conn.last_insert_rowid()
}

pub fn inserir_porta(conn: &Connection, host_id: i64, porta: u16, esperado: bool) {
    conn.execute(
        "INSERT INTO portas (host_id, porta, esperado) VALUES (?1, ?2, ?3)",
        rusqlite::params![host_id, porta, esperado as i64],
    )
    .unwrap_or_else(|e| {
        crate::log::erro(&format!("erro ao inserir porta: {}", e));
        panic!();
    });
}

pub struct Host {
    pub id: i64,
    pub dominio: Option<String>,
    pub ip: Option<String>,
    pub status: String,
}

pub struct Porta {
    pub id: i64,
    pub host_id: i64,
    pub porta: u16,
    pub status: String,
    pub esperado: bool
}

pub fn buscar_hosts(conn: &Connection) -> Vec<Host> {
    let mut stmt  = conn.prepare("SELECT id, dominio, ip, status FROM hosts")
        .unwrap_or_else(|e| {
            crate::log::erro(&format!("erro ao preparar query de hosts: {}", e));
            panic!();
        });

    stmt.query_map([], |row| {
        Ok(Host {
            id: row.get(0)?,
            dominio: row.get(1)?,
            ip: row.get(2)?,
            status: row.get(3)?,
        })
    })
    .unwrap_or_else(|e| {
        crate::log::erro(&format!("erro ao executar query de hosts: {}", e));
        panic!();
    })
    .filter_map(|r| r.ok())
    .collect()
}

pub fn buscar_portas_do_host(conn: &Connection, host_id: i64) -> Vec<Porta> {
    let mut stmt = conn.prepare(
        "SELECT id, host_id, porta, status, esperado FROM portas WHERE host_id = ?1"
    )
    .unwrap_or_else(|e| {
        crate::log::erro(&format!("erro ao preparar query de portas: {}", e));
        panic!();
    });

    stmt.query_map(rusqlite::params![host_id], |row| {
        Ok(Porta {
            id: row.get(0)?,
            host_id: row.get(1)?,
            porta: row.get(2)?,
            status: row.get(3)?,
            esperado: row.get::<_, i64>(4)? != 0
        })
    })
    .unwrap_or_else(|e| {
        crate::log::erro(&format!("erro ao executar query de portas: {}", e));
        panic!();
    })
    .filter_map(|r| r.ok())
    .collect()
}

pub fn atualizar_status_porta(conn: &Connection, id: i64, status: &str, agora: i64) {
    let coluna_horario = if status == "ativo" {
        "ultimo_horario_ativo"
    } else {
        "ultimo_horario_inativo"
    };

    let coluna_tempo = if status == "ativo" {
        "tempo_ativo"
    } else {
        "tempo_inativo"
    };

    let sql = format!(
        "UPDATE portas SET status = ?1, checado_em = ?2, {} = ?2, {} = {} + ?3 WHERE id = ?4", 
        coluna_horario, coluna_tempo, coluna_tempo
    );

    conn.execute(
        &sql,
        rusqlite::params![status, agora, 1i64, id],
    )
    .unwrap_or_else(|e| {
        crate::log::erro(&format!("erro ao atualizar status da porta {}: {}", id, e));
        panic!();
    });
}

pub fn atualizar_status_host(conn: &Connection, id: i64, status: &str, agora: i64) {
    let coluna_horario = if status == "ativo" {
        "ultimo_horario_ativo"
    } else {
        "ultimo_horario_inativo"
    };

    let coluna_tempo = if status == "ativo" {
        "tempo_ativo"
    } else {
        "tempo_inativo"
    };

    let sql = format!(
        "UPDATE hosts SET status = ?1, checado_em = ?2, {} = ?2, {} = {} + ?3 WHERE id = ?4",
        coluna_horario, coluna_tempo, coluna_tempo
    );

    conn.execute(
        &sql,
        rusqlite::params![status, agora, 1i64, id]
    )
    .unwrap_or_else(|e| {
        crate::log::erro(&format!("erro ao atualizar status do host {}: {}", id, e));
        panic!();
    }); 
}

pub fn remover_hosts(conn: &Connection, id: i64) {
    // remove as portas primeiro por causa da FOREIGN KEY
    conn.execute("DELETE FROM portas WHERE host_id = ?1", rusqlite::params![id])
        .unwrap_or_else(|e| {
            crate::log::erro(&format!("erro ao remover portas do host {}: {}", id, e));
            panic!();
        });

    conn.execute("DELETE FROM hosts WHERE id = ?1", rusqlite::params![id])
        .unwrap_or_else(|e| {
            crate::log::erro(&format!("erro ao remover host {}: {}", id, e));
            panic!();
        })
}

pub fn abrir_conexao() -> Connection {
    let caminho = caminho_banco();
    Connection::open(&caminho)
        .unwrap_or_else(|e| {
            crate::log::erro(&format!("erro ao abrir conexão auxiliar: {}", e));
            panic!();
        })
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
//   o operador ? propaga o erro pra quem chamou a função — só funciona
//   em funções que retornam Result ou Option.
//   unwrap_or_else trata na hora — útil quando a função retorna o valor
//   puro (Connection, PathBuf) e você quer logar antes de encerrar.
//
// Option vs Result
//   Result<T, E> → operação que pode falhar com um erro descrito (E)
//   Option<T>    → valor que pode simplesmente não existir (None)
//   dirs::data_dir() retorna Option porque não há "erro" — o OS
//   simplesmente pode não ter esse caminho definido.
//   por isso o closure do unwrap_or_else recebe () e não |e|:
//   não tem erro pra capturar, só ausência de valor.
// abrir_conexao()
//   abre uma conexão nova com o mesmo banco sem criar tabelas.
//   necessário porque rusqlite::Connection não implementa Send —
//   não pode ser compartilhada entre threads diretamente.
//   cada thread que precisa acessar o banco abre sua própria conexão.
//   o SQLite suporta múltiplas conexões simultâneas no mesmo arquivo
//   com segurança — usa WAL (Write-Ahead Logging) internamente
//   pra evitar conflito entre leituras e escritas concorrentes.
//   usada pelo ipc.rs que roda em thread separada do daemon principal.
//
// por que não Arc<Mutex<Connection>>?
//   seria possível — envolve a Connection num Mutex pra acesso exclusivo
//   e num Arc pra compartilhar entre threads com contagem de referência.
//   mas isso serializa todos os acessos — enquanto o IPC usa o banco,
//   o daemon espera, e vice-versa.
//   duas conexões independentes é mais simples e mais performático
//   pra esse caso de uso onde as operações são rápidas e pouco frequentes.