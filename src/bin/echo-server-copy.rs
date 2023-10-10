use tokio::{
  io::{self, AsyncReadExt, AsyncWriteExt},
  net::TcpListener,
};

const PORT: u32 = 6379;
const LOCALHOST: &str = "127.0.0.1";

#[tokio::main]
async fn main() -> io::Result<()> {
  let server_socket_addr = format!("{LOCALHOST}:{PORT}");
  let listener = TcpListener::bind(&server_socket_addr).await?;
  // let (mut rd, mut wr) = io::split(listener);

  println!("listening on {server_socket_addr}");

  loop {
    let (mut socket, addr) = listener.accept().await?;
    dbg!(&socket, &addr);

    tokio::spawn(async move {
      let mut buf = vec![0; 1024];
      loop {
        match socket.read(&mut buf).await {
          Ok(0) => break,
          Ok(n) => {
            println!("{}", &String::from_utf8_lossy(&buf[..n]));
            if socket.write_all(&buf[..n]).await.is_err() {
              return;
            }
          }
          Err(e) => {
            dbg!("ERROR: {:?}", e);
            return;
          }
        }
      }
    });
  }

  // Ok(())
}
