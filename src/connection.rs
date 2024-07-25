use bytes::{Buf, BytesMut};
use std::error::Error;
use std::io::Cursor;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufWriter},
    net::TcpStream,
};

use mini_redis::Frame;

pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut,
    cursor: usize,
}

impl Connection {
    pub fn new(stream: TcpStream) -> Connection {
        Connection {
            stream: BufWriter::new(stream),
            buffer: BytesMut::with_capacity(4096),
            cursor: 0,
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>, Box<dyn Error>> {
        loop {
            if let Some(frame) = self.parse_frame()? {
                return Ok(Some(frame));
            }

            if self.buffer.len() == self.cursor {
                self.buffer.resize(self.cursor * 2, 0);
            }
            let n = self.stream.read(&mut self.buffer[self.cursor..]).await?;

            if 0 == n {
                if self.cursor == 0 {
                    return Ok(None);
                } else {
                    return Err("connection reset by peer".into());
                }
            } else {
                self.cursor += n;
            }
        }
    }

    pub async fn write_frame(&mut self, frame: &Frame) -> Result<(), Box<dyn Error>> {
        match frame {
            Frame::Simple(val) => {
                self.stream.write_u8(b'+').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Error(val) => {
                self.stream.write_u8(b'-').await?;
                self.stream.write_all(val.as_bytes()).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Integer(val) => {
                self.stream.write_u8(b':').await?;
                self.write_decimal(*val).await?;
            }
            Frame::Null => self.stream.write_all(b"$-1\r\n").await?,

            Frame::Bulk(val) => {
                let len = val.len();

                self.stream.write_u8(b'$').await?;
                self.write_decimal(len as u64).await?;
                self.stream.write_all(val).await?;
                self.stream.write_all(b"\r\n").await?;
            }
            Frame::Array(_) => unimplemented!(),
        }
        self.stream.flush().await?;
        Ok(())
    }

    async fn write_decimal(&mut self, _val: u64) -> Result<(), Box<dyn Error>> {
        unimplemented!()
    }
}

#[allow(dead_code)]
impl Connection {
    fn parse_frame(&mut self) -> Result<Option<Frame>, Box<dyn Error>> {
        let mut buff = Cursor::new(&self.buffer[..]);

        match Frame::check(&mut buff) {
            Ok(_) => {
                let len = buff.position() as usize;

                buff.set_position(0);

                let frame = Frame::parse(&mut buff)?;

                self.buffer.advance(len);

                Ok(Some(frame))
            }
            Err(mini_redis::frame::Error::Incomplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
