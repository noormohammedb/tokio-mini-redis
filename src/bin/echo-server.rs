use std::error::Error;
use tokio::io;
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut stream, s_addr) = listener.accept().await?;

        tokio::spawn(async move {
            println!("Accepted Connection: {:?}", s_addr);
            let (mut tx, mut rx) = TcpStream::split(&mut stream);
            if let Err(e) = io::copy(&mut tx, &mut rx).await {
                eprintln!("failed to copy\t{:?}", e);
            }
            // let mut buff = vec![0; 1024];
            // loop {
            //     match tx.read(&mut buff).await {
            //         Ok(0) => return,
            //         Err(e) => eprintln!("failed to copy\t{:?}", e),
            //         Ok(n) => {
            //             println!("{}", String::from_utf8_lossy(&buff[..n]));
            //             rx.write_all(&buff[..n]).await.unwrap();
            //         }
            //     };
            // }
        });
    }
}
