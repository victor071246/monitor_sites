use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

pub enum StatusConexao{
    Ativo,
    Inativo
}

pub fn checar_porta(endereco: &str, porta: u16) -> StatusConexao {
    let destino = format!("{}:{}", endereco, porta);
    let timeout = Duration::from_secs(5);

    let addr = destino
        .to_socket_addrs()
        .unwrap_or_else(|e| {
            crate::log::erro(&format!("erro ao resolver endereço {}: {}", destino, e));
            panic!();
        })
        .next()
        .unwrap_or_else(|| {
            crate::log::erro(&format!("nenhum endereço resolvido para {}", destino));
            panic!();
        });
    
    match TcpStream::connect_timeout(&addr, timeout) {
        Ok(_) => StatusConexao::Ativo,
        Err(_) => StatusConexao::Inativo
    }
}

// ============================================================
// NOTAS DE ESTUDO
// ============================================================
//
// ToSocketAddrs
//   trait da stdlib que resolve uma string "host:porta" em um ou mais
//   SocketAddr (IP + porta já resolvidos).
//   faz resolução DNS via OS — o mesmo mecanismo que o browser usa.
//   retorna um iterador porque um domínio pode ter múltiplos IPs (CDN).
//   .next() pega o primeiro — suficiente pra checar conectividade.
//
// Duration::from_secs(5)
//   timeout de 5 segundos por tentativa.
//   sem timeout, a thread pode ficar pendurada minutos em host morto.
//
// TcpStream::connect_timeout(&SocketAddr, Duration)
//   tenta estabelecer conexão TCP com tempo limite.
//   recebe SocketAddr (IP já resolvido) — por isso o ToSocketAddrs antes.
//   Ok(_)  → handshake TCP bem sucedido, porta está ouvindo
//   Err(_) → conexão recusada, timeout, ou host inacessível
//
// match vs if let
//   match é preferível quando você trata ambos os casos explicitamente.
//   deixa claro que Ativo e Inativo são os dois únicos resultados.
//
// por que não registrar o Err no log aqui?
//   host inativo é evento normal num monitor — não é erro do programa.
//   o scanner.rs vai decidir se registra como aviso ou só atualiza o banco.