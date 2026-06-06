use std::net::TcpStream;
use std::io::{Write, Read};
use std::fs;
use colored::Colorize;
use serde::Deserialize;

#[derive(Deserialize)]
struct Config {
    monitor: Monitor
}

#[derive(Deserialize)]
struct Monitor {
    hosts: Vec<String>,
    port: u16,
}

fn check_host(host: &str, port: u16) {
    println!("{}", format!("[ >> ] Disparando GET em {}:{} sem TLS", host, port).purple());

    let mut stream = match  TcpStream::connect(format!("{}:{}", host, port)) {
        Ok(s) => s,
        Err(e) => {
            println!("{}", format!("[ !! ] {} -> falha na conexão: {}", host, e).red());
            return;
        }
        
    };

    let request = format!("GET / HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", host);
    stream.write_all(request.as_bytes()).unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();

    let status_line = response.lines().next().unwrap_or("sem respose");
    let status_code = status_line.split_whitespace().nth(1).unwrap_or("???");

    match status_code {
        "200" => println!("{}", format!("[ OK ] {} -> {}", host, status_line).green()),
        "301" | "302" => println!("{}", format!("[ >> ] {} -> {} (redirect", host, status_line).yellow()),
        "404" => println!("{}", format!("[ XX ] {} -> {}", host, status_line).red()),
        "500" | "503" => println!("{}", format!("[ !! ] {} -> {}", host, status_line).red().bold()),
        _ => println!("{}", format!("[ ?? ] {} -> {}", host, status_line).yellow())
    }
}

fn main() {
    let content = fs::read_to_string("config.toml").expect("config.toml não encontrado");
    let config: Config = toml::from_str(&content).expect("Erro ao parsear config.toml");

    for host in &config.monitor.hosts {
        check_host(host, config.monitor.port);
    }

}