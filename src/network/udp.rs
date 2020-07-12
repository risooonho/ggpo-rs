use crate::network::udp_msg::UdpMsg;

use async_compression::futures::{bufread::ZstdDecoder, write::ZstdEncoder};
use async_dup::Arc;
use async_net::UdpSocket;
use bytes::{Bytes, BytesMut};
use log::{error, info};
use smol::Async;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use thiserror::Error;

pub trait UdpCallback {
    fn on_msg(&self, _from: &SocketAddr, _msg: &UdpMsg, _len: usize) {}
}

#[derive(Debug, Error)]
pub enum UdpError {
    #[error("Socket unbound/unitialized.")]
    SocketUninit,
    #[error("Session callbacks uninitialized.")]
    CallbacksUninit,
    #[error("IO Error")]
    Io {
        #[from]
        source: std::io::Error,
    },
    #[error("Bincode (de)serialization Error")]
    Bincode {
        #[from]
        source: bincode::Error,
    },
}

async fn create_socket(socket_address: SocketAddr, retries: usize) -> std::io::Result<UdpSocket> {
    for port in (socket_address.port() as usize)..(socket_address.port() as usize) + retries + 1 {
        match UdpSocket::bind(SocketAddr::new(socket_address.ip(), port as u16)).await {
            Ok(soc) => {
                info!("Udp bound to port: {}.\n", port);
                return Ok(soc);
            }
            Err(error) => {
                error!("Failed to bind to socket. {:?}", error);
            }
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::Other,
        format!(
            "failed to bind socket after {} successive retries.",
            retries
        ),
    ))
}

pub struct Udp<'callbacks, Callbacks>
where
    Callbacks: UdpCallback,
{
    // Network transmission information
    socket: Option<UdpSocket>,

    // state management
    callbacks: Option<&'callbacks mut Callbacks>,
    // poll: Option<&'poll mut Poll>,
}

impl<'callbacks, Callbacks> Default for Udp<'callbacks, Callbacks>
where
    Callbacks: UdpCallback,
{
    fn default() -> Self {
        Udp {
            socket: None,
            callbacks: None,
            // poll: None,
        }
    }
}

impl<'callbacks, Callbacks> Udp<'callbacks, Callbacks>
where
    Callbacks: UdpCallback,
{
    pub const fn new() -> Self {
        Udp {
            socket: None,
            callbacks: None,
            // poll: None,
        }
    }
    pub async fn init(
        &mut self,
        port: u16,
        callbacks: &'callbacks mut Callbacks,
    ) -> Result<(), UdpError> {
        self.callbacks = Some(callbacks);
        info!("binding udp socket to port {}.\n", port);
        self.socket = Some(
            create_socket(
                SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port),
                3,
            )
            .await?,
        );

        Ok(())
    }

    pub async fn send_to(
        &mut self,
        msg: Arc<UdpMsg>,
        destination: &[SocketAddr],
    ) -> Result<(), UdpError> {
        let serialized = bincode::serialize(&(*msg))?;
        let resp = self
            .socket
            .as_ref()
            .ok_or(UdpError::SocketUninit)?
            .send_to(&serialized, destination)
            .await?;

        let peer_addr = self
            .socket
            .as_ref()
            .ok_or(UdpError::SocketUninit)?
            .peer_addr()?;
        info!(
            "sent packet length {} to {}:{} (resp:{}).\n",
            serialized.len(),
            peer_addr.ip(),
            peer_addr.port(),
            resp
        );
        Ok(())
    }

    pub async fn on_loop_poll(&self, _cookie: i32) -> Result<bool, UdpError> {
        let mut recv_buf = BytesMut::new();
        let (len, recv_address) = self
            .socket
            .as_ref()
            .ok_or(UdpError::SocketUninit)?
            .recv_from(recv_buf.as_mut())
            .await?;

        let msg: UdpMsg = bincode::deserialize(recv_buf.as_mut())?;
        self.callbacks
            .as_ref()
            .ok_or(UdpError::CallbacksUninit)?
            .on_msg(&recv_address, &msg, len);
        Ok(true)
    }
}
