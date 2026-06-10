use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use chrono::Local;

pub enum Nivel {
    Info,
    Aviso,
    Erro
}

fn caminho_log() -> PathBuf {
    let mut path = dirs::data_dir()
        .unwrap_or(PathBuf::from("."));
    path.push("monitor_sites");
    std::fs::create_dir_all(&path).ok();
    path.push("monitor.log");
    path
}

fn escrever(nivel: Nivel, mensagem: &str) {
    let agora = Local::now().format("%d/%m/%Y %H:%M:%S").to_string();

    let nivel_str = match nivel {
        Nivel::Info => "INFO",
        Nivel::Aviso => "AVISO",
        Nivel::Erro => "ERRO"
    };

    let linha = format!("{} [{}] {}\n", agora, nivel_str, mensagem);

    let mut arquivo = OpenOptions::new()
        .create(true)
        .append(true)
        .open(caminho_log())
        .unwrap_or_else(|e| panic!("erro ao abrir log: {}", e));

    arquivo.write_all(linha.as_bytes()).ok();
}

pub fn info(msg: &str) { escrever(Nivel::Info, msg)}
pub fn aviso(msg: &str) {escrever(Nivel::Aviso, msg);}
pub fn erro(msg: &str) {escrever(Nivel::Erro, msg);}


// ============================================================
// NOTAS DE ESTUDO
// ============================================================
//
// OBJETIVO DESTE MÓDULO:
// Registrar eventos do daemon em arquivo de texto persistente.
// Diferente do banco (que guarda estado atual), o log guarda
// histórico linear de tudo que aconteceu — útil pra debugar
// e auditar comportamento ao longo do tempo.
//
// ONDE O LOG É SALVO:
//   Windows → C:\Users\<user>\AppData\Roaming\monitor_sites\monitor.log
//   Linux   → ~/.local/share/monitor_sites/monitor.log
//   macOS   → ~/Library/Application Support/monitor_sites/monitor.log
//
// -------------------------------------------------------
// use chrono::Local
//   módulo do crate chrono que representa o fuso horário local
//   da máquina. Local::now() retorna a data e hora atual já
//   convertida pro fuso configurado no sistema operacional.
//   Se a máquina está em BRL, já sai no horário correto.
//   Não precisamos do chrono-tz porque não estamos forçando
//   um timezone específico — usamos o do sistema.
//
// -------------------------------------------------------
// enum Nivel
//   tipo soma resolvido inteiramente em tempo de compilação.
//   Não aloca memória dinâmica — o compilador transforma cada
//   variante num inteiro discriminante internamente.
//   O match vira jump table direta — sem overhead em runtime.
//   Type safety: impossível passar valor inválido, erro de
//   compilação se tentar usar variante inexistente.
//   Diferente de &str "INFORMAÇÃO" que é ponteiro com tamanho
//   variável e só estoura erro em runtime se digitado errado.
//
// -------------------------------------------------------
// fn caminho_log() -> PathBuf
//   PathBuf é um caminho de arquivo que pode ser construído
//   em partes. Multiplataforma — usa \ no Windows e / no Unix.
//   dirs::data_dir() resolve o diretório de dados do usuário
//   por plataforma sem hardcode.
//   .unwrap_or(PathBuf::from(".")) → se data_dir() falhar,
//   usa o diretório atual como fallback em vez de crashar.
//   create_dir_all cria o diretório e todos os pais necessários
//   se já existir não faz nada. Equivalente a mkdir -p no shell.
//   .ok() descarta o erro — se não conseguir criar o diretório,
//   o erro vai aparecer na hora de abrir o arquivo de qualquer forma.
//
// -------------------------------------------------------
// fn escrever(nivel: Nivel, mensagem: &str)
//   função privada — só chamada pelas funções públicas abaixo.
//   Local::now().format("%d/%m/%Y %H:%M:%S") formata a data
//   no padrão brasileiro: 08/06/2026 02:45:00
//   OpenOptions é um builder — você configura o comportamento
//   antes de abrir o arquivo:
//     .create(true)  → cria o arquivo se não existir
//     .append(true)  → adiciona no final, nunca sobrescreve
//     .open(caminho) → só aqui o arquivo é aberto de verdade,
//                      as chamadas anteriores só configuram
//   .unwrap_or_else(|e| panic!(...)) → se não conseguir abrir
//   o arquivo de log, crasha com mensagem clara. Diferente do
//   .ok() do create_dir_all — aqui não tem fallback útil.
//   arquivo.write_all(linha.as_bytes()) converte a String pra
//   &[u8] e escreve todos os bytes de uma vez no arquivo.
//   .ok() no final descarta o erro de escrita silenciosamente —
//   se o log falhar não queremos derrubar o daemon inteiro.
//
// -------------------------------------------------------
// funções públicas: informacao / aviso / erro
//   wrappers finos que chamam escrever com o nível correto.
//   São as únicas funções expostas pro resto do projeto.
//   O resto do código nunca chama escrever() diretamente.
//
// -------------------------------------------------------
// EXEMPLO DE SAÍDA NO ARQUIVO:
//   08/06/2026 02:45:00 [INFO] host example.com adicionado
//   08/06/2026 02:45:10 [INFO] example.com:80 respondeu
//   08/06/2026 02:46:10 [ERRO] example.com:443 falha na conexão
//   08/06/2026 02:47:10 [AVISO] porta 8080 aberta inesperadamente em 1.2.3.4