// use std::io::Cursor;

use bytes::{Buf, BytesMut};
use mini_redis::{Frame, Result};
use tokio::{
  io::{self, AsyncWriteExt, BufWriter},
  net::TcpStream,
};

pub struct Connection {
  stream: BufWriter<TcpStream>,
  // buffer: Vec<u8>,
  buffer: BytesMut,
  // cursor: usize,
}
impl Connection {
  pub fn new(stream: TcpStream) -> Self {
    Self {
      stream: BufWriter::new(stream),
      // buffer: vec![0; 4096],
      buffer: BytesMut::with_capacity(4096),
      // cursor: 0,
    }
  }

  fn parse_frame(&self) -> Result<Option<Frame>> {
    // let mut buf = Cursor::new(&self.buffer[..]);

    // match Frame::check(&mut buf) {
    //   Ok(_) => {
    //     let len = buf.position() as usize;
    //     buf.set_position(0);

    //     let frame = Frame::parse(&mut buf)?;
    //     buf.advance(len);
    //     return Ok(Some(frame));
    //   }
    //   Err(Error::Incomplete) => {
    //     return Ok(None);
    //   }
    //   Err(e) => {
    //     dbg!(&e);
    //     Err(e.into())
    //   }
    // }
    Ok(None)
  }

  pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
    loop {
      // if let Some(frame) = self.parse_frame()? {
      //   return Ok(Some(frame));
      // }
      // if self.buffer.len() == self.cursor {
      //   self.buffer.resize(self.cursor * 2, 0);
      // }
      // let n = self
      //   .stream
      //   .read(&mut self.buffer[self.cursor..])
      //   .await?;

      // if 0 == n {
      //   if self.buffer.is_empty() {
      //     return Ok(None);
      //   } else {
      //     return Err("Connection reset by peer".into());
      //   }
      // } else {
      //   self.cursor += n;
      // }
      dbg!("read_frame Loooooooooooooooooooooooooooooooop");
    }
  }

  // pub async fn write_frame(self, res: &Frame) -> mini_redis::Result<()> {
  pub async fn write_frame(&mut self, frame: &Frame) -> io::Result<()> {
    match frame {
      //
      Frame::Simple(fr_str) => {
        self.stream.write_u8(b'+').await?;
        self.stream.write_all(fr_str.as_bytes()).await?;
        self.stream.write_all(b"\r\n").await?;
      }
      Frame::Error(fr_str) => {
        self.stream.write_u8(b'-').await?;
        self.stream.write_all(fr_str.as_bytes()).await?;
        self.stream.write_all(b"\r\n").await?;
      }
      Frame::Integer(data) => {
        self.stream.write_u8(b':').await?;
        // self.write_decimal(*data).await?;
      }
      Frame::Null => {
        self.stream.write_all(b"$-1\r\n").await?;
      }
      Frame::Bulk(data) => {
        let len = data.len();

        self.stream.write_u8(b'$').await?;
        // self.write_decimal(len as u64).await?;
        self.stream.write_all(data).await?;
        self.stream.write_all(b"\r\n").await?;
      }
      Frame::Array(data_vec) => {
        unimplemented!()
      }
    }

    let _ = self.stream.flush().await;
    Ok(())
  }

  pub async fn write_decimal(self, data: u64) -> () {
    //
  }
}
