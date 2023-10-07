use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

const PORT: u32 = 6379;
const LOCALHOST: &str = "127.0.0.1";

#[tokio::main]
async fn main() -> () {
  #[derive(Debug)]
  enum Command {
    Get {
      key: String,
      resp: Responder<Option<Bytes>>,
    },
    Set {
      key: String,
      val: Bytes,
      resp: Responder<()>,
    },
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
        Get { key, resp } => {
          let res = client.get(&key).await;
          resp.send(res).unwrap();
        }
        Set { key, val, resp } => {
          let res = client.set(&key, val).await;
          resp.send(res).unwrap();
        }
      }
    }
  });

  let tx2 = tx.clone();

  let t1 = tokio::spawn(async move {
    let (resp_tx, resp_rx) = oneshot::channel();
    let cmd = Command::Get {
      key: "foo".into(),
      resp: resp_tx,
    };
    let _ = tx.send(cmd).await.unwrap();

    let res = resp_rx.await;
    println!("GOT = {:?}", res);
  });

  let t2 = tokio::spawn(async move {
    let (resp_tx, resp_rx) = oneshot::channel();
    let cmd = Command::Set {
      key: "foo".into(),
      val: "bar".into(),
      resp: resp_tx,
    };
    let _ = tx2.send(cmd).await.unwrap();

    let res = resp_rx.await;
    println!("GOT = {:?}", res);
  });

  t1.await.unwrap();
  t2.await.unwrap();
  let _ = conn_manager.await;
}
