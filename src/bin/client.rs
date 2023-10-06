use mini_redis::client;

const PORT: u32 = 6379;
const LOCALHOST: &str = "127.0.0.1";

#[tokio::main]
async fn main() -> () {
  let server_socket_addr = format!("{LOCALHOST}:{PORT}");
  let mut client_01 = client::connect(&server_socket_addr)
    .await
    .expect("Connection failed to {server_socket_addr}");
  let mut client_02 = client::connect(&server_socket_addr)
    .await
    .expect("Connection failed to {server_socket_addr}");

  let t1 = tokio::spawn(async move {
    let _ = client_01.get("foo").await;
  });

  let t2 = tokio::spawn(async move {
    let _ = client_02.set("foo", "bar".into()).await;
  });

  t1.await.unwrap();
  t2.await.unwrap();
}
