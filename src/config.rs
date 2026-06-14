use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
pub struct Config {
    pub daemon: Daemon,
}

#[derive(Deserialize)]
pub struct Daemon {
    pub intervalo: u64,
    pub socket: String,
}

pub fn carregar () -> Config {
    let caminho = caminho_config();
    garantir_config(&caminho);
    let conteudo = std::fs::read_to_string(&caminho)
        .unwrap_or_else(|e| {
            crate::log::erro(&format!("erro ao ler config.toml: {}", e));
            panic!();
        });
    toml::from_str(&conteudo)
        .unwrap_or_else(|e| {
            crate::log::erro(&format!("erro ao parsear config.toml: {}", e));
            panic!();
        })
}

fn caminho_config() -> PathBuf {
    let mut path = dirs::data_dir()
        .unwrap_or_else(|| {
            crate::log::erro("não foi possível resolver o diretório de dados");
            panic!();
        });
    path.push("monitor_sites");
    path.push("config.toml");
    path
}

fn garantir_config(caminho: &PathBuf) {
    if !caminho.exists() {
        let conteudo_padrao = "[daemon]\nintervalo = 15\nsocket = \"\"\n";
        std::fs::write(caminho, conteudo_padrao)
            .unwrap_or_else(|e| {
                crate::log::erro(&format!("erro ao criar config.toml padrão: {}", e));
                panic!();
            });
        crate::log::info("config.toml criado com valores padrão");
    }
}

// ============================================================
// NOTAS DE ESTUDO
// ============================================================
//
// #[derive(Deserialize)]
//   macro que gera automaticamente o código de desserialização
//   para a struct. o serde mapeia chave do TOML → campo da struct
//   pelo nome. se o nome não bater, retorna erro no from_str.
//
// pub struct Config / pub struct Daemon
//   duas structs espelhando a hierarquia do TOML:
//   [daemon] no TOML → campo daemon: Daemon na Config
//   as structs precisam ser pub pra outros módulos acessarem
//   os campos também precisam ser pub pelo mesmo motivo
//
// u64 para intervalo
//   segundos entre checks — nunca negativo, não precisa de sinal.
//   u64 cobre até ~584 bilhões de anos. u32 seria suficiente mas
//   u64 é o tipo padrão de Duration::from_secs() no Rust.
//
// caminho_config()
//   o config.toml fica no mesmo diretório do banco:
//   Linux   → ~/.local/share/monitor_sites/config.toml
//   Windows → C:\Users\<user>\AppData\Roaming\monitor_sites\config.toml
//   macOS   → ~/Library/Application Support/monitor_sites/config.toml
//   separado do binário — o usuário pode editar sem recompilar.
//
// toml::from_str(&conteudo)
//   desserializa a string TOML na struct Config.
//   retorna Result<Config, toml::de::Error>
//   o unwrap_or_else registra o erro no log antes de encerrar.