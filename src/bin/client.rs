use bytes::Bytes;
use mini_redis::client;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
enum MyCommand {
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

type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

#[tokio::main]
async fn main() {
    let (tx_main, mut rx_main) = mpsc::channel(32);
    let tx2_main = tx_main.clone();

    let manager = tokio::spawn(async move {
        let mut client = client::connect(("127.0.0.1", 6379)).await.unwrap();

        while let Some(cmd) = rx_main.recv().await {
            match cmd {
                MyCommand::Get { key, resp } => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                MyCommand::Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = MyCommand::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };
        if tx_main.send(cmd).await.is_err() {
            eprintln!("connection task shutdown");
            return;
        };

        let res = resp_rx.await.unwrap();
        println!("GOT (Get) = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = MyCommand::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: resp_tx,
        };
        if tx2_main.send(cmd).await.is_err() {
            eprintln!("connection task shutdown");
            return;
        }
        let res = resp_rx.await.unwrap();
        println!("SET (Set) = {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
