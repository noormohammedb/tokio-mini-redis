use tokio::{
  io::{self, AsyncReadExt, AsyncWriteExt},
  net::TcpStream,
};

const PORT: u32 = 6379;
const LOCALHOST: &str = "127.0.0.1";

#[tokio::main]
async fn main() -> io::Result<()> {
  let server_socket_addr = format!("{LOCALHOST}:{PORT}");
  let socket = TcpStream::connect(&server_socket_addr).await?;
  let (mut rd, mut wr) = io::split(socket);

  tokio::spawn(async move {
    wr.write_all(b"Hello\r\n").await?;
    wr.write_all(b"world\r\n").await?;

    Ok::<_, io::Error>(())
  });

  let mut buf = vec![0; 128];

  loop {
    let n = rd.read(&mut buf).await?;

    if n == 0 {
      break;
    }

    println!("GOT {:?}", String::from_utf8_lossy(&buf[..n]));
  }

  Ok(())
}
