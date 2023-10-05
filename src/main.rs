use std::collections::HashMap;

use mini_redis::{
  Command::{self, Get, Set},
  Connection, Frame,
};
use tokio::net::{TcpListener, TcpStream};

const PORT: u32 = 6379;
const LOCALHOST: &str = "127.0.0.1";

#[tokio::main]
async fn main() -> () {
  let lisn_socket_addr = format!("{LOCALHOST}:{PORT}");
  let listener = TcpListener::bind(&lisn_socket_addr).await.unwrap();

  println!("server listening on: {lisn_socket_addr}");

  loop {
    let (socket, _) = listener.accept().await.unwrap();
    tokio::spawn(async move {
      process(socket).await;
    });
  }
}

async fn process(sockt: TcpStream) -> () {
  let mut db = HashMap::new();

  let mut connection = Connection::new(sockt);

  while let Some(frame) = connection.read_frame().await.unwrap() {
    println!("GOT: {:?}", frame);
    let response = match Command::from_frame(frame).unwrap() {
      Set(cmd) => {
        db.insert(cmd.key().to_string(), cmd.value().to_vec());
        Frame::Simple("OK".to_string())
      }
      Get(cmd) => {
        if let Some(value) = db.get(cmd.key()) {
          Frame::Bulk(value.clone().into())
        } else {
          Frame::Null
        }
      }
      cmd => panic!("unimplemented {:?}", cmd),
    };

    // let response = Frame::Error("unimplemented".to_string());
    connection.write_frame(&response).await.unwrap();
  }
}
