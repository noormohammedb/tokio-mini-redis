use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use bytes::Bytes;
use mini_redis::{
  Command::{self, Get, Set},
  Connection, Frame,
};
use tokio::net::{TcpListener, TcpStream};

type DB = Arc<Mutex<HashMap<String, Bytes>>>;

const PORT: u32 = 6379;
const LOCALHOST: &str = "127.0.0.1";

#[tokio::main]
async fn main() -> () {
  let lisn_socket_addr = format!("{LOCALHOST}:{PORT}");
  let listener = TcpListener::bind(&lisn_socket_addr).await.unwrap();

  let db: DB = Arc::new(Mutex::new(HashMap::new()));

  println!("server listening on: {lisn_socket_addr}");

  loop {
    let (socket, _) = listener.accept().await.unwrap();

    let db = db.clone();
    tokio::spawn(async move {
      process(socket, db).await;
    });
  }
}

async fn process(sockt: TcpStream, db: DB) -> () {
  let mut connection = Connection::new(sockt);

  while let Some(frame) = connection.read_frame().await.unwrap() {
    println!("GOT: {:?}", frame);
    let response = match Command::from_frame(frame).unwrap() {
      Set(cmd) => {
        db.lock()
          .expect("cant lock the db mutex")
          .insert(cmd.key().to_string(), cmd.value().clone());

        Frame::Simple("OK".to_string())
      }
      Get(cmd) => {
        if let Some(value) = db
          .lock()
          .expect("cant lock the db mutex when reading")
          .get(cmd.key())
        {
          Frame::Bulk(value.clone())
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
