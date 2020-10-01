use httparse;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub(crate) struct HptsConfig {
    pub socks5_addr: SocketAddr,
}

pub(crate) struct HptsContext {
    pub config: Arc<HptsConfig>,
    pub socket: TcpStream,
    pub buf: [u8; 4096],
    pub pos: usize,
}

impl HptsContext {
    pub fn new(config: Arc<HptsConfig>, socket: TcpStream) -> Self {
        HptsContext {
            config,
            socket,
            buf: [0; 4096],
            pos: 0,
        }
    }
}

pub(crate) async fn hpts_bridge(ctx: HptsContext) {
    let mut ctx = ctx;
    let buf = &mut ctx.buf;
    let socket = &mut ctx.socket;

    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let n = socket
        .read(&mut buf[ctx.pos..])
        .await
        .expect("failed to read data from socket");
    if n == 0 {
        return;
    }
    ctx.pos += n;
    println!("n: {:?}, data: {}", n, &std::str::from_utf8(buf).unwrap());
    if !req.parse(buf).unwrap().is_complete() {
        eprintln!("incomplete http request");
        return;
    }
    dbg!(req);
}
