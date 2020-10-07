use futures::future::try_join;
use httparse;
use log::{debug, error, trace};
use std::error::Error;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use tokio::io;
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
    pub resend: bool,
}

impl HptsContext {
    pub fn new(config: Arc<HptsConfig>, socket: TcpStream) -> Self {
        HptsContext {
            config,
            socket,
            buf: [0; 4096],
            pos: 0,
            resend: true,
        }
    }
}

pub(crate) async fn hpts_bridge(ctx: HptsContext) -> Result<(), Box<dyn Error>> {
    let mut ctx = ctx;
    let buf = &mut ctx.buf;
    let socket = &mut ctx.socket;

    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let n = socket.read(&mut buf[ctx.pos..]).await?;
    if n == 0 {
        return Ok(());
    }
    ctx.pos += n;
    if !req.parse(buf).unwrap().is_complete() {
        error!("incomplete http request");
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "incomplete http request",
        )));
    }

    let mut port = 80;
    if req.method.unwrap().to_lowercase() == "connect" {
        // When request a HTTPS site, client will send CONNECT method HTTP request
        // In this case, we don't need to resend buffer(which is in HTTP format),
        // and just forward in TCP level
        ctx.resend = false;
        port = 443;
        trace!("https");
    }

    let mut socks5_buf = [0; 1024];
    let mut socks5_stream = TcpStream::connect(&ctx.config.socks5_addr).await?;
    socks5_stream.write_all(&[05, 02, 00, 01]).await?;
    // skip check for now
    socks5_stream.read(&mut socks5_buf).await?;
    let mut host = "";
    for i in 0..16 {
        let h = headers[i];
        if h.name.to_lowercase() == "host" {
            host = std::str::from_utf8(h.value).unwrap();
            let host_port = host.split(":").collect::<Vec<&str>>();
            host = host_port[0];
            if host_port.len() > 1 {
                port = host_port[1].parse().unwrap();
            }
            break;
        }
    }

    debug!("proxy to: {}:{}", host, port);

    let n = build_socks5_cmd(&mut socks5_buf, &host, port);
    trace!("cmd: {:?}", &socks5_buf[0..n]);

    socks5_stream.write_all(&&socks5_buf[0..n]).await?;
    // check OK
    socks5_stream.read(&mut socks5_buf).await?;
    // write the first packet
    if ctx.resend {
        socks5_stream.write_all(&ctx.buf).await?;
    } else {
        // write 200 back to client
        ctx.socket.write_all(b"HTTP/1.1 200 OK\r\n\r\n").await?;
    }

    // start buffering
    let (mut ri, mut wi) = ctx.socket.split();
    let (mut ro, mut wo) = socks5_stream.split();

    let client_to_server = async {
        io::copy(&mut ri, &mut wo).await?;
        wo.shutdown().await
    };

    let server_to_client = async {
        io::copy(&mut ro, &mut wi).await?;
        wi.shutdown().await
    };

    try_join(client_to_server, server_to_client).await?;
    Ok(())
}

pub fn build_socks5_cmd(buf: &mut [u8], host: &str, port: u16) -> usize {
    let maybe_ip: Result<IpAddr, _> = host.parse();
    buf[0] = 5;
    buf[1] = 1; // CONNECT
    buf[2] = 0;
    match maybe_ip {
        Ok(ip) => match ip {
            IpAddr::V4(ipv4) => {
                buf[3] = 1;
                let ip_octets = ipv4.octets();
                for i in 0..4 {
                    buf[4 + i] = ip_octets[i];
                }
                buf[8] = (port >> 8) as u8;
                buf[9] = port as u8;
                return 10;
            }
            IpAddr::V6(ipv6) => {
                buf[3] = 4;
                let ip_octets = ipv6.octets();
                for i in 0..16 {
                    buf[4 + i] = ip_octets[i];
                }
                buf[20] = (port >> 8) as u8;
                buf[21] = port as u8;
                return 22;
            }
        },
        Err(_) => {
            // domain
            buf[3] = 3;
            buf[4] = host.len() as u8;
            let data = host.as_bytes();
            for i in 0..host.len() {
                buf[5 + i] = data[i];
            }
            buf[5 + host.len()] = (port >> 8) as u8;
            buf[5 + host.len() + 1] = port as u8;
            return 5 + host.len() + 1 + 1;
        }
    }
}
