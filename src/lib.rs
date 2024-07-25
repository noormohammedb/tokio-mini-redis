use bytes::Bytes;

mod connection;
pub use connection::Connection;

pub enum Frame {
    Simple(String),
    Error(String),
    Integer(u64),
    Bulk(Bytes),
    Null,
    Array(Vec<Frame>),
}
