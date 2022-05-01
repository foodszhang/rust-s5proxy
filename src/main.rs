mod config;
mod error;
mod server;
extern crate pretty_env_logger;
use clap::Parser;
use std::path::PathBuf;

extern crate log;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long)]
    ip: Option<String>,
    #[clap(short, long)]
    port: Option<u16>,
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    config: Option<PathBuf>,
}

#[tokio::main]
pub async fn main() {
    pretty_env_logger::init();
    let args = Args::parse();
    if let Some(config_path) = args.config.as_deref() {
        let s = server::Server::from_file(config_path).unwrap();
        println!("error: {}", s.listen().await.unwrap_err());
        return;
    }
    if args.ip == None || args.port == None {
        println!("must use ip, port or configfile to start server");
        return;
    }
    let ip = args.ip.unwrap();
    let port = args.port.unwrap();
    let s = server::Server { ip, port };
    println!("error: {}", s.listen().await.unwrap_err());
}
