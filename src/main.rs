mod config;
mod checker;

fn main() {
    let config = config::carregar();
    for host in &config.monitor.hosts {
        checker::check_host(host, config.monitor.port);
    }
}