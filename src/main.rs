use mini_redis::{Connection, Frame};
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
    process(socket).await;
  }
}

async fn process(sockt: TcpStream) -> () {
  let mut connection = Connection::new(sockt);

  if let Some(frame) = connection.read_frame().await.unwrap() {
    println!("GOT: {:?}", frame);

    let response = Frame::Error("unimplemented".to_string());
    connection.write_frame(&response).await.unwrap();
  }
}
