use interprocess::local_socket::{
    GenericFilePath, ListenerOptions, Stream, ToFsName, traits::ListenerExt
};
use std::io::{BufRead, BufReader, Write};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Comando{
    comando: String,
    dominio: Option<String>,
    ip: Option<String>,
    host_id: Option<i64>,
    porta: Option<u16>,
}

#[derive(Serialize)]
struct Resposta {
    ok: bool,
    dados: Option<String>,
    erro: Option<String>,
}

pub fn iniciar(tx: std::sync::mpsc::Sender<bool>) {
    let nome = "/tmp/monitor_sites.sock"
        .to_fs_name::<GenericFilePath>()
        .unwrap_or_else(|e|{
            tx.send(false).ok();
            crate::log::erro(&format!("erro ao criar nome do socket: {}", e));
            panic!();
        });

    std::fs::remove_file("/tmp/monitor_sites.sock").ok();

    let listener = ListenerOptions::new()
        .name(nome)
        .create_sync()
        .unwrap_or_else(|e| {
            tx.send(false).ok();
            crate::log::erro(&format!("erro ao criar socket IPC: {}", e));
            panic!();
        });

    tx.send(true).ok();
    crate::log::info("ipc escutando conexões");

    for stream in listener.incoming().flatten() {
        processar_conexao(stream);
    }
}

fn processar_conexao(stream: Stream) {
    let conn = crate::db::abrir_conexao();
    let mut reader = BufReader::new(&stream);
    let mut linha = String::new();

    if reader.read_line(&mut linha).is_err() {
        return;
    }

    let resposta = match serde_json::from_str::<Comando>(&linha) {
        Ok(cmd) => executar_comando(&conn, cmd),
        Err(e) => Resposta {
            ok: false,
            dados: None,
            erro: Some(format!("json inválido: {}", e))
        }
    };

    let json = serde_json::to_string(&resposta).unwrap_or_default();
    let mut writer = &stream;
    writer.write_all(json.as_bytes()).ok();
    writer.write_all(b"\n").ok();
}

fn executar_comando(conn: &rusqlite::Connection, cmd: Comando) -> Resposta {
    match cmd.comando.as_str() {
        "adicionar_host" => {
            let id = crate::db::inserir_host(
                conn,
                cmd.dominio.as_deref(),
                cmd.ip.as_deref(),
            );
            crate::log::info(&format!("host adicionado id={}", id));
            Resposta { ok: true, dados: Some(id.to_string()), erro: None}
        }
        "adicionar_porta" => {
            match (cmd.host_id, cmd.porta) {
                (Some(host_id), Some(porta)) => {
                    crate::db::inserir_porta(conn, host_id, porta, true);
                    crate::log::info(&format!("porta {} adicionado ao host {}", porta, host_id));
                    Resposta { ok: true, dados: None, erro: None}
                }
                _ => Resposta {
                    ok: false,
                    dados: None,
                    erro: Some("host_id e porta são obrigatórios".to_string())
                }
            }
        }
        "listar_hosts" => {
            let hosts = crate::db::buscar_hosts(conn);
            let lista: Vec<String> = hosts.iter()
                .map(|h| {
                    let endereco = h.dominio.as_deref()
                    .or(h.ip.as_deref())
                    .unwrap_or("sem endereço");
                    format!("{}::{}::{}", h.id, endereco, h.status)
                })
                .collect();
            Resposta { ok: true, dados: Some(lista.join("|")), erro: None}
        }
        _ => Resposta {
            ok: false,
            dados: None,
            erro: Some(format!("comando desconhecido: {}", cmd.comando))
        }
    }
}

// ============================================================
// NOTAS DE ESTUDO
// ============================================================
//
// OBJETIVO DESTE MÓDULO:
//   Canal de comunicação entre o daemon e a GUI.
//   O daemon roda em background — a GUI manda comandos JSON
//   via socket local e o daemon responde com JSON.
//
// GenericFilePath
//   tipo de nome de socket que usa caminho real no filesystem.
//   Linux/macOS → arquivo em /tmp/monitor_sites.sock
//   diferente de GenericNamespaced que usa namespace abstrato
//   e não tem suporte garantido em todos os sistemas Linux.
//   to_fs_name::<GenericFilePath>() converte a string do caminho
//   num tipo que o interprocess entende.
//
// ListenerOptions::new().name(nome).create_sync()
//   cria o socket e começa a escutar conexões.
//   create_sync é a versão bloqueante — não usa async/await.
//   se o arquivo /tmp/monitor_sites.sock já existir de uma
//   execução anterior, pode falhar — o daemon deve remover
//   o arquivo antes de criar o listener se necessário.
//
// listener.incoming().flatten()
//   incoming() retorna um iterador infinito de conexões entrantes.
//   cada item é Result<Stream, Error>.
//   flatten() descarta automaticamente os Err — conexões que
//   falharam no accept não derrubam o loop inteiro.
//
// BufReader + read_line
//   lê uma linha completa do stream até encontrar \n.
//   o protocolo é: um JSON por linha, terminado com \n.
//   simples e robusto — fácil de implementar em qualquer cliente.
//   BufReader bufferiza internamente — mais eficiente que
//   ler byte a byte diretamente do stream.
//
// serde_json::from_str::<Comando>
//   deserializa a string JSON na struct Comando.
//   o turbofish ::<Comando> especifica o tipo alvo.
//   campos Option<T> aceitam ausência no JSON — viram None.
//   campo "comando" é obrigatório — ausente causa Err.
//
// serde_json::to_string(&resposta)
//   serializa a struct Resposta em string JSON.
//   unwrap_or_default() retorna string vazia se falhar —
//   improvável já que Resposta tem só tipos simples.
//
// processar_conexao abre conexão própria com o banco
//   rusqlite::Connection não é Send — não pode ser
//   compartilhada entre threads.
//   cada conexão IPC abre sua própria Connection via
//   crate::db::abrir_conexao() e fecha ao fim da função.
//
// #[derive(Deserialize)] em Comando
//   serde gera código de desserialização em tempo de compilação.
//   zero overhead em runtime — não usa reflection.
//
// #[derive(Serialize)] em Resposta
//   serde gera código de serialização em tempo de compilação.
//   Option<T> None vira null no JSON.
//   Option<T> Some(v) vira o valor v diretamente.
//
// PROTOCOLO IPC — FORMATO DAS MENSAGENS:
//   entrada (GUI → daemon):
//     {"comando":"adicionar_host","dominio":"example.com","ip":null}
//     {"comando":"adicionar_porta","host_id":1,"porta":443}
//     {"comando":"listar_hosts"}
//
//   saída (daemon → GUI):
//     {"ok":true,"dados":"1","erro":null}
//     {"ok":false,"dados":null,"erro":"mensagem de erro"}