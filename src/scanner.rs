use std::net::TcpStream;
use std::time::Duration;
use std::net::ToSocketAddrs;

pub enum StatusConexao {
    Ativo,
    Inativo
}

pub fn checar(host: &str, porta: u16) -> StatusConexao {
    let endereco = format!("{}:{}", host, porta);

    let mut addrs = match endereco.to_socket_addrs() {
        Ok(a) => a,
        Err(e) => {
            crate::log::erro(&format!("erro ao resolver endereço {}:{} - {}", host, porta, e));
            return  StatusConexao::Inativo;
        }
    };

    let addr = match addrs.next() {
        Some(a) => a,
        None => {
            crate::log::erro(&format!("nenhum endereço resolvido para {}:{}", host, porta));
            return StatusConexao::Inativo
        }
    };

    let timeout = Duration::from_secs(5);

    match TcpStream::connect_timeout(&addr, timeout) {
        Ok(_) => StatusConexao::Ativo,
        Err(_) => StatusConexao::Inativo
    }

}

// ============================================================
// NOTAS DE ESTUDO
// ============================================================
//
// TcpStream::connect_timeout(&addr, timeout)
//   variante do connect que respeita um tempo máximo de espera.
//   sem timeout, uma porta filtrada por firewall pode travar
//   o programa por vários minutos esperando resposta que nunca vem.
//   Duration::from_secs(5) → desiste após 5 segundos sem resposta.
//
// ToSocketAddrs
//   trait da stdlib que converte strings como "example.com:80"
//   em SocketAddr (endereço IP + porta já resolvidos).
//   faz resolução DNS internamente — transforma domínio em IP.
//   retorna um iterador porque um domínio pode resolver em
//   múltiplos IPs (balanceador de carga, IPv4 + IPv6).
//   usamos .next() pra pegar só o primeiro endereço resolvido.
//
// pub enum StatusConexao
//   tipo soma com duas variantes — Ativo ou Inativo.
//   resolvido em tempo de compilação, sem alocação dinâmica.
//   o daemon vai usar esse retorno pra decidir o que gravar
//   no banco e se precisa registrar mudança de estado no log.
//
// por que não retornar bool?
//   bool funciona mas é menos expressivo — true/false não diz
//   nada sobre o domínio. StatusConexao::Ativo é autoexplicativo.
//   também facilita expandir no futuro com mais variantes como
//   StatusConexao::Timeout ou StatusConexao::ErroResolucaoDns.