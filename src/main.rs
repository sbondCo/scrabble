use futures::{SinkExt, StreamExt};
use log::*;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::{Error, Message::Text, Result};

#[derive(Serialize, Deserialize, Debug)]
struct Request {
  job: u32,
}

async fn accept_connection(peer: SocketAddr, stream: TcpStream) {
  if let Err(e) = handle_connection(peer, stream).await {
    match e {
      Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
      err => error!("Error processing connection: {}", err),
    }
  }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream) -> Result<()> {
  let mut ws_stream = accept_async(stream).await.expect("Failed to accept");

  info!("New WebSocket connection: {}", peer);

  while let Some(payload) = ws_stream.next().await {
    let payload = payload?;
    if payload.is_text() || payload.is_binary() {
      info!("{}", &payload.to_string());

      let msg: Request = serde_json::from_str(&payload.to_string()).unwrap();
      let resp: &str;

      match msg.job {
        1 => resp = "hi",
        _ => resp = "e",
      }

      ws_stream.send(Text(resp.to_string())).await?;
    }
  }

  Ok(())
}

#[tokio::main]
async fn main() {
  env_logger::init();

  let addr = "127.0.0.1:8080";
  let listener = TcpListener::bind(&addr).await.expect("Can't listen");
  info!("Listening on: {}", addr);

  while let Ok((stream, _)) = listener.accept().await {
    let peer = stream
      .peer_addr()
      .expect("connected streams should have a peer address");
    info!("Peer address: {}", peer);

    tokio::spawn(accept_connection(peer, stream));
  }
}
