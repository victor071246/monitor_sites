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
        if endereco.is_empty() {
            crate::log::aviso(&format!("hosy id {} sem domínio e sem ip, pulando", host_id));
            continue;
        }

        let portas = crate::db::buscar_portas_do_host(conn, *host_id);
        let agora = agora_unix();
        let mut alguma_inativa = false;

        for porta in &portas {
            let status = match crate::checker::checar_porta(endereco, porta.porta) {
                crate::checker::StatusConexao::Ativo => "ativo",
                crate::checker::StatusConexao::Inativo => "inativo",
            };

            if status != porta.status {
                crate::log::info(&format!(
                    "{}:{} {} => {}",
                    endereco, porta.porta, porta.status, status
                ));
            }

            if status == "inativo" {
                alguma_inativa = true;
            }

            crate::db::atualizar_status_porta(conn, porta.id, status, agora);
        }

        let status_host = if alguma_inativa { "inativo" } else { "ativo" };
        crate::db::atualizar_status_host(conn, *host_id, status_host, agora);
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

// ============================================================
// NOTAS DE ESTUDO
// ============================================================
//
// loop {}
//   loop infinito — não tem condição de saída.
//   o daemon roda indefinidamente até o processo ser encerrado
//   pelo sistema operacional ou pelo usuário.
//
// thread::sleep(Duration::from_secs(intervalo))
//   pausa a thread pelo tempo configurado sem consumir CPU.
//   o processo fica em estado de espera no kernel — o OS acorda
//   quando o timer expira.
//
// agora_unix()
//   SystemTime::now() retorna o tempo atual do sistema.
//   duration_since(UNIX_EPOCH) calcula segundos desde 01/01/1970.
//   as_secs() retorna u64, cast pra i64 porque SQLite usa INTEGER com sinal.
//   unwrap_or_default() retorna 0 se falhar — impossível na prática.
//
// buscar_hosts — retorna tupla em vez de struct
//   aqui usamos Vec<(i64, Option<String>, Option<String>)> em vez
//   de Vec<Host> porque é uma query local ao daemon — não precisa
//   expor a struct completa. o db.rs já tem buscar_hosts com struct
//   pra uso externo.
//
// dominio.as_deref().or(ip.as_deref()).unwrap_or("")
//   as_deref() converte Option<String> → Option<&str> sem mover o valor.
//   .or() encadeia — se dominio for None, tenta ip.
//   .unwrap_or("") garante &str vazio como fallback — verificado logo depois.
//
// *host_id
//   host_id é &i64 porque o for itera por referência (&hosts).
//   o * dereferencia pra passar o i64 por valor pra função.
//
// alguma_inativa
//   flag que acumula o estado geral do host.
//   se qualquer porta estiver inativa, o host é marcado como inativo.
//   lógica conservadora — host só é ativo se todas as portas responderem.
//
// status != porta.status
//   só loga quando o estado muda — não a cada check.
//   evita poluir o log com eventos repetidos quando tudo está estável.