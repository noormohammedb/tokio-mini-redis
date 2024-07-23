use std::error::Error;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stream = TcpStream::connect(("127.0.0.1", 8080)).await?;

    // let (mut rd, mut wr) = io::split(stream);
    // let (mut rd, mut wr) = TcpStream::split(&mut stream);
    let (mut rd, mut wr) = stream.into_split();

    tokio::spawn(async move {
        wr.write_all(b"hello\r\n").await?;
        wr.write_all(b"world\r\n").await?;
        Ok::<(), io::Error>(())
    });

    let mut buff = vec![0; 128];

    loop {
        let n = rd.read(&mut buff).await?;

        if n == 0 {
            break;
        }
        let data = &buff[..n];
        println!("GOT {:?}", String::from_utf8_lossy(data));
    }

    Ok(())
}
