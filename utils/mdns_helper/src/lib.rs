pub use dns_parser_revived::*;
use std::{
    io::{Error as IOErr, ErrorKind},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    time::{Duration, Instant},
};
use thiserror::Error;

const MULTICAST_ADDR: Ipv4Addr = Ipv4Addr::new(224, 0, 0, 251);
const MULTICAST_PORT: u16 = 5353;
const BUFF_SIZE: usize = 4096;

pub struct MdnsFinder {
    socket: UdpSocket,
}

impl MdnsFinder {
    /// 创建mDNS查询器并加入多播组
    pub fn new() -> Result<Self, DNSErr> {
        let socket = UdpSocket::bind("0.0.0.0:0")?; // 创建udp
        socket.join_multicast_v4(&MULTICAST_ADDR, &Ipv4Addr::UNSPECIFIED)?; // 加入多播组, 是固定地址
        socket.set_multicast_loop_v4(true)?; // 开启多播循环
        socket.set_ttl(255)?;
        Ok(Self { socket })
    }

    /// 发送mDNS查询
    pub fn send_query(self, query: &str) -> Result<Self, DNSErr> {
        let query = self.build_query(query);
        let dest = SocketAddr::V4(SocketAddrV4::new(MULTICAST_ADDR, MULTICAST_PORT));
        self.socket.send_to(&query, dest)?;
        Ok(self)
    }

    fn build_query(&self, query: &str) -> Vec<u8> {
        let mut bd = Builder::new_query(1, true);
        bd.add_question(query, false, QueryType::PTR, QueryClass::IN);
        bd.build().unwrap_or_else(|x| x)
    }

    /// 在时间内接收mDNS响应
    pub fn recv_resp<T, F>(&mut self, timeout: Duration, hanle: F) -> Result<Vec<T>, DNSErr>
    where
        F: Fn(SocketAddr, Packet<'_>) -> Option<T>,
    {
        let start = Instant::now();
        let mut buff = [0u8; BUFF_SIZE];
        let mut res = vec![];

        while start.elapsed() < timeout {
            let remaining = timeout - start.elapsed();
            self.socket.set_read_timeout(Some(remaining))?;

            match self.socket.recv_from(&mut buff) {
                Ok((size, src)) => {
                    let data = buff[..size].to_vec();
                    let pack = Self::parse(&data)?;
                    if let Some(x) = hanle(src, pack) {
                        res.push(x);
                    }
                }
                Err(e) => match e.kind() {
                    ErrorKind::WouldBlock => continue, // 超时，继续等待
                    ErrorKind::TimedOut => break,      // 超时，退出循环
                    _ => return Err(e.into()),
                },
            };
        }

        Ok(res)
    }

    fn parse(buff: &[u8]) -> Result<Packet<'_>, DNSErr> {
        let pk = Packet::parse(buff)?;
        let resp_code = pk.header.response_code;
        if resp_code != ResponseCode::NoError {
            return Err(resp_code.into());
        }

        Ok(pk)
    }
}

#[derive(Error, Debug)]
pub enum DNSErr {
    #[error("{0}")]
    ParseError(#[from] dns_parser_revived::Error),
    #[error("{0}")]
    ResError(#[from] ResponseCode),
    #[error("{0}")]
    IOError(#[from] IOErr),
}
