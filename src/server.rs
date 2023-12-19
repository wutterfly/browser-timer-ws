use data::Data;
use err::{Error, MessageError};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use message::TimerMessage;
use ping::PingPong;
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::Duration,
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Message, Result},
    WebSocketStream,
};

use crate::{
    data::{self, DataHolder, Distributer},
    err, message, ping,
    time::now,
};

pub struct Server {
    /// Specifies in what interval(sec) ping messages should be sent.
    ping_interval: u64,
}

impl Server {
    pub const fn new(ping_interval: u64) -> Self {
        Self { ping_interval }
    }

    /// Starts the websocket server.
    pub async fn start_server(
        &self,
        data: Distributer,
        port: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let addr = SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

        // Create the event loop and TCP listener we'll accept connections on.
        let try_socket = TcpListener::bind(&addr).await?;
        let listener = try_socket;

        log::info!("Listening on: {addr}");

        while let Ok((stream, _)) = listener.accept().await {
            let peer = stream
                .peer_addr()
                .expect("connected streams should have a peer address");

            let holder = data.new_connection();

            tokio::spawn(Self::accept_connection(
                peer,
                stream,
                holder,
                self.ping_interval,
            ));
        }

        Ok(())
    }

    /// Accepts a connection and makes inital handshake.
    async fn accept_connection(
        peer: SocketAddr,
        stream: TcpStream,
        data: DataHolder,
        ping_interval: u64,
    ) {
        // accept connection and handle handshake
        let ws_stream = match accept_async(stream).await {
            Ok(s) => {
                log::debug!("New WebSocket connection: {peer}");
                s
            }
            Err(e) => {
                log::error!("{e}");
                return;
            }
        };

        // handle incoming messages
        if let Err(e) = Self::handle_connection(ws_stream, data, ping_interval).await {
            log::error!("{e}");
        } else {
            log::debug!("connection closed: {peer}");
        }
    }

    /// Receives messages and sends ping requests.
    async fn handle_connection(
        ws_stream: WebSocketStream<TcpStream>,
        mut data: DataHolder,
        ping_interval: u64,
    ) -> Result<(), Error> {
        // split connection into sender and receiver
        let (mut ws_sender, mut ws_receiver) = ws_stream.split();

        // create interval for requesting rount-trip-time messages
        let mut interval = tokio::time::interval(Duration::from_secs(ping_interval));
        let mut ping = PingPong::new();

        loop {
            tokio::select! {
                msg = ws_receiver.next() => {
                    if let Some(msg) = msg {
                        // check if closed
                        if Self::handle_message(msg?, &mut ping,&mut ws_sender, &mut data).await? {
                            break;
                        }
                    }
                    // connection closed
                    else {
                        break;
                    }
                }
                _ = interval.tick() => {

                    if let Err((c, elapsed)) = ping.close(){
                        log::warn!("current ping still open: [{c}] - {elapsed}ms");
                    }

                    match ping.next() {
                        Ok(i) => {
                            ws_sender
                            .send(Message::Text(TimerMessage::Ping { i }.as_string()))
                            .await?;
                        },
                        Err(_) => unreachable!(),
                    }

                }
            }
        }

        Ok(())
    }

    /// Handles individual messages.
    /// Returns true if connection was closed, else returns false.
    ///
    /// # Errors
    /// Returns any error that occures while handling messages(network, parsing).
    async fn handle_message(
        msg: Message,
        ping: &mut PingPong,
        ws_sender: &mut SplitSink<WebSocketStream<TcpStream>, Message>,
        data: &mut DataHolder,
    ) -> Result<bool, Error> {
        if let Message::Text(txt) = msg {
            let msg = TimerMessage::try_from(txt.as_str())?;

            match msg {
                TimerMessage::Pong { i } => match ping.finish(i) {
                    Ok(rtt) => {
                        ws_sender
                            .send(Message::Text(TimerMessage::Rtt { rtt }.as_string()))
                            .await?;
                        data.update_rtt(rtt);
                    }
                    Err(Some(c)) => log::warn!("no matching counter: {c}"),
                    Err(None) => log::warn!("no ping requested"),
                },

                TimerMessage::Data { key, key_code, typ } => {
                    let timestamp = now();

                    #[allow(clippy::cast_possible_truncation)]
                    let relativ = (timestamp - data.first_timestamp()) as u64;

                    data.push(Data {
                        key,
                        timestamp: relativ,
                        typ,
                        key_code,
                    })?;
                }

                _ => unreachable!(),
            }

            return Ok(false);
        } else if msg.is_close() {
            return Ok(true);
        }

        Err(MessageError::UnexpectedMessageTyp.into())
    }
}
