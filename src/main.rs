mod config;
mod db;
mod log;
mod scanner;
mod checker;
mod daemon;
mod ipc;

use colored::Colorize;
use std::thread;
use std::io::Write;
use std::time::Duration;
use std::sync::mpsc;

fn main() {
    let config = config::carregar();
    let conn = db::inicializar();
    
    let (tx, rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        ipc::iniciar(tx);
    });
    let protocolo_iniciado = rx.recv().unwrap_or(false);

    if protocolo_iniciado {
        println!("{}", "[ OK ] ipc iniciado".purple());
    } else {
        println!("{}", "[ !! ] ipc falhou".purple());
    }

    thread::spawn(||{
        let frames = [
            "[ ** ] daemon rodando    ",
            "[ ** ] daemon rodando .  ",
            "[ ** ] daemon rodando .. ",
            "[ ** ] daemon rodando ..."
        ];
        let mut i = 0;
        loop {
            print!("\r{}", frames[i % 4].purple());
            std::io::stdout().flush().ok();
            i += 1;
            thread::sleep(Duration::from_millis(500));
        }
    });

    daemon::iniciar(&conn, config.daemon.intervalo);
}

// ============================================================
// NOTAS DE ESTUDO
// ============================================================
//
// mod config / db / log / scanner / checker / daemon
//   declara os módulos do projeto — o compilador procura
//   cada um em src/<nome>.rs e os compila juntos.
//   sem a declaração aqui, o módulo não existe pro programa.
//
// thread::spawn(|| { ... })
//   cria uma nova thread que roda em paralelo com a thread principal.
//   recebe uma closure — bloco de código que roda na nova thread.
//   a thread do spawn roda a animação enquanto a thread principal
//   roda o daemon::iniciar — os dois ao mesmo tempo.
//   se a thread principal encerrar, a thread do spawn é derrubada
//   automaticamente — não precisa gerenciar manualmente.
//
// frames[i % 4]
//   i % 4 garante que o índice sempre fica entre 0 e 3.
//   quando i chega em 4, volta pra 0 — ciclo infinito pelos frames.
//   evita índice fora do array sem precisar resetar i manualmente.
//
// print! vs println!
//   println! adiciona \n no final — pula a linha.
//   print! não pula — permite sobrescrever a mesma linha com \r.
//   \r é carriage return — move o cursor pro início da linha atual
//   sem descer. a próxima escrita sobrescreve o que estava antes.
//   resultado: animação na mesma linha em vez de scroll infinito.
//
// stdout().flush()
//   o terminal bufferiza a saída — acumula antes de exibir.
//   flush() força exibir imediatamente sem esperar o buffer encher.
//   sem flush, a animação pode não aparecer ou aparecer em blocos.
//   .ok() descarta o erro — se o flush falhar, a animação só
//   fica um pouco atrasada, não é crítico.
//
// .purple()
//   método do crate colored aplicado em &str.
//   adiciona escape codes ANSI de cor roxa no terminal.
//   funciona em Linux e macOS nativamente.
//   no Windows requer terminal com suporte ANSI (Windows Terminal, VSCode).
//
// daemon::iniciar(&conn, config.daemon.intervalo)
//   chamada bloqueante — nunca retorna enquanto o daemon roda.
//   por isso vem depois do thread::spawn — se viesse antes,
//   o spawn nunca seria executado.