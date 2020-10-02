use clap::{App, Arg};
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::TcpListener;

use env_logger::Builder;
use log::LevelFilter;
use log::{debug, error, info};

mod hpts;
use hpts::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut logger_builder = Builder::new();

    let matches = App::new("hpts")
        .version("0.0")
        .author("Yongsheng Xu")
        .about("Turn your socks proxy into http proxy")
        .arg(
            Arg::with_name("socks")
                .short("s")
                .help("specify your socks proxy host, default: 127.0.0.1:1080")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .help("specify the listening port of http proxy server, default: 8080")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches();
    let socks: SocketAddr = matches
        .value_of("socks")
        .unwrap_or("127.0.0.1:1080")
        .parse()
        .unwrap();

    let config = Arc::new(HptsConfig { socks5_addr: socks });

    let port: u16 = matches.value_of("port").unwrap_or("8080").parse().unwrap();
    let http_proxy_sock = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    let level = match matches.occurrences_of("v") {
        0 => LevelFilter::Error,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        3 | _ => LevelFilter::Trace,
    };

    logger_builder.filter_level(level);
    logger_builder.init();

    info!("http server listening on port {}", port);

    let mut listener = TcpListener::bind(http_proxy_sock).await?;
    loop {
        let (socket, _addr) = listener.accept().await?;
        debug!("accept from client: {}", _addr);
        let ctx = HptsContext::new(config.clone(), socket);
        match hpts_bridge(ctx).await {
            Ok(()) => {}
            Err(err) => error!("{}", err),
        };
    }
}
