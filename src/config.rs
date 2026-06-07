use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub monitor: Monitor,
}

#[derive(Deserialize)]
pub struct Monitor {
    pub hosts: Vec<String>,
    pub port: u16,
}

pub fn carregar() -> Config {
    let content = std::fs::read_to_string("config.toml")
        .expect("config.toml não encontrado");
    toml::from_str(&content).expect("Erro ao parsear config.toml")
}