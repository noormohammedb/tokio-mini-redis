use bytes::Bytes;
use mini_redis::client;
use tokio::sync::mpsc;

const PORT: u32 = 6379;
const LOCALHOST: &str = "127.0.0.1";

#[tokio::main]
async fn main() -> () {
  #[derive(Debug)]
  enum Command {
    Get { key: String },
    Set { key: String, val: Bytes },
  }
  let server_socket_addr = format!("{LOCALHOST}:{PORT}");

  let (tx, mut rx) = mpsc::channel::<Command>(32);

  let conn_manager = tokio::spawn(async move {
    let mut client = client::connect(&server_socket_addr)
      .await
      .expect("Connection failed to {server_socket_addr}");

    while let Some(cmd) = rx.recv().await {
      use Command::{Get, Set};

      match cmd {
        Get { key } => {
          let _ = client.get(&key).await;
        }
        Set { key, val } => {
          let _ = client.set(&key, val).await;
        }
      }
    }
  });

  let tx2 = tx.clone();

  let t1 = tokio::spawn(async move {
    let cmd = Command::Get { key: "foo".into() };
    let _ = tx.send(cmd).await.unwrap();
  });

  let t2 = tokio::spawn(async move {
    let cmd = Command::Set {
      key: "foo".into(),
      val: "bar".into(),
    };
    let _ = tx2.send(cmd).await.unwrap();
  });

  t1.await.unwrap();
  t2.await.unwrap();
  let _ = conn_manager.await;
}
