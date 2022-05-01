use crate::config;
use crate::error::{S5ErrorType, S5Exception, UnExpectedError};
use byteorder::{BigEndian, ByteOrder};
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::path::Path;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
#[derive(Debug)]
pub struct Server {
    pub ip: String,
    pub port: u16,
}
type SResult = Result<Server, UnExpectedError>;
type NResult = Result<(), UnExpectedError>;
fn protocol_error() -> UnExpectedError {
    return UnExpectedError::S5Error(S5Exception {
        error_type: S5ErrorType::ProtocolError,
        error_message: "Only Support Noauthorization".to_string(),
    });
}
impl Server {
    pub fn from_file(filename: &Path) -> SResult {
        let config = config::Config::from_file(filename)?;
        Server::from_config(&config)
    }
    pub fn from_config(config: &config::Config) -> SResult {
        let ip = config.ip.clone().unwrap_or("127.0.0.1".to_string());
        let port = config.port.unwrap_or(3001);
        Ok(Server { ip, port })
    }
    async fn validate_protocol(stream: &mut TcpStream) -> NResult {
        let mut buffer: [u8; 80] = [0; 80];
        let len = stream.read_exact(&mut buffer[0..2]).await?;
        if len != 2 {
            return Err(protocol_error());
        }
        if buffer[0] != 0x05 {
            return Err(protocol_error());
        }
        let methods_len = buffer[1] as usize;
        stream.read_exact(&mut buffer[0..methods_len]).await?;
        let bufs = buffer[0..methods_len].iter().filter(|x| **x == 0x00);
        if bufs.count() == 0 {
            stream.write(&[0x05, 0xFF]).await?;
        }
        stream.write(&[0x05, 0x00]).await?;

        Ok(())
    }
    async fn proxy(from: &mut TcpStream, to: &mut TcpStream) -> NResult {
        let (mut from_rx, mut from_tx) = from.split();
        let (mut to_rx, mut to_tx) = to.split();
        let t1 = tokio::io::copy(&mut to_rx, &mut from_tx);
        let t2 = tokio::io::copy(&mut from_rx, &mut to_tx);
        tokio::try_join!(t1, t2)?;
        Ok(())
    }
    async fn request(stream: &mut TcpStream) -> NResult {
        let mut request_header: [u8; 4] = [0; 4];
        stream.read_exact(&mut request_header[0..4]).await?;
        if request_header[0] != 0x05 {
            return Err(protocol_error());
        }
        let mut tg_addr: Option<SocketAddr> = std::default::Default::default();

        match request_header[3] {
            0x01 => {
                let mut addr_buffer: [u8; 6] = [0; 6];
                stream.read_exact(&mut addr_buffer).await?;
                tg_addr = Some(SocketAddr::V4(SocketAddrV4::new(
                    Ipv4Addr::from(BigEndian::read_u32(&addr_buffer[0..4])),
                    BigEndian::read_u16(&request_header[4..6]),
                )));
            }
            0x03 => {
                let domain_len = stream.read_u8().await? as usize;
                let mut addr_buffer: [u8; 258] = [0; 258];
                stream
                    .read_exact(&mut addr_buffer[0..domain_len + 2])
                    .await?;
                let domain = std::str::from_utf8(&addr_buffer[0..domain_len])?;
                let port = BigEndian::read_u16(&addr_buffer[domain_len..domain_len + 2]);

                for addr in tokio::net::lookup_host((domain, port)).await? {
                    tg_addr = Some(addr);
                    break;
                }
            }
            0x04 => {
                let mut addr_buffer: [u8; 18] = [0; 18];
                stream.read_exact(&mut addr_buffer).await?;
                tg_addr = Some(SocketAddr::V6(SocketAddrV6::new(
                    Ipv6Addr::from(BigEndian::read_u128(&addr_buffer[0..16])),
                    BigEndian::read_u16(&addr_buffer[16..18]),
                    0,
                    0,
                )));
            }
            _ => {
                return Err(protocol_error());
            }
        }
        if tg_addr == None {
            stream
                .write_all(&[0x05, 0x08, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
                .await?;
            return Err(protocol_error());
        }
        let tg_addr = tg_addr.unwrap();
        log::trace!("Ipv4Addr: {:?}", &tg_addr);
        match request_header[1] {
            0x01 => {
                let mut target_stream = TcpStream::connect(&tg_addr).await?;
                stream
                    .write_all(&[0x05, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
                    .await?;
                Server::proxy(stream, &mut target_stream).await?;
            }
            _ => {
                return Err(protocol_error());
            }
        }

        Ok(())
    }

    pub async fn process(mut stream: TcpStream) {
        match Server::validate_protocol(&mut stream).await {
            Ok(()) => log::trace!("Protol Ok"),
            Err(e) => {
                log::warn!("Proxy Not Ok, reason:{}", e);
            }
        }
        match Server::request(&mut stream).await {
            Ok(()) => log::trace!("Over Connection"),
            Err(e) => {
                log::warn!("Proxy Not Ok, reason:{}", e);
            }
        }
    }
    pub async fn listen(self) -> NResult {
        let listener = TcpListener::bind(format!("{}:{}", self.ip, self.port)).await?;
        log::info!("Bind on {}", format!("{}:{}", self.ip, self.port));
        loop {
            let (stream, addr) = listener.accept().await?;
            tokio::spawn(async move {
                log::info!("Start a new connection from {}", addr);
                Server::process(stream).await;
            });
        }
    }
}
