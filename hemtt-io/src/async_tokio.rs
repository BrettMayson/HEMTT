use tokio::io::{AsyncReadExt, AsyncWriteExt};

use std::io::Result;

use async_trait::async_trait;

#[async_trait]
pub trait ReadExt: AsyncReadExt + std::marker::Unpin + std::marker::Send {
    async fn read_cstring(&mut self) -> Result<String>;
    async fn read_compressed_int(&mut self) -> Result<u32>;
}

#[async_trait]
impl<T: AsyncReadExt + std::marker::Unpin + std::marker::Send> ReadExt for T {
    async fn read_cstring(&mut self) -> Result<String> {
        let mut byte = vec![0u8; 1];
        let mut bytes: Vec<u8> = Vec::new();
        loop {
            self.read_exact(&mut byte).await?;
            if byte[0] == 0 {
                break;
            }
            bytes.push(byte[0]);
        }
        Ok(String::from_utf8(bytes).unwrap())
    }

    async fn read_compressed_int(&mut self) -> Result<u32> {
        let mut byte = vec![0u8; 1];
        let mut result: u32 = 0;
        let mut i = 0;
        loop {
            self.read_exact(&mut byte).await?;
            let b: u32 = byte[0].into();
            result |= (b & 0x7f) << (i * 7);
            i += 1;
            if byte[0] < 0x80 {
                break;
            }
        }
        Ok(result)
    }
}

#[async_trait]
pub trait WriteExt: AsyncWriteExt + std::marker::Unpin + std::marker::Send {
    async fn write_cstring(&mut self, s: &[u8]) -> Result<()>;
    async fn write_compressed_int(&mut self, x: u32) -> Result<usize>;
}

#[async_trait]
impl<T: AsyncWriteExt + std::marker::Unpin + std::marker::Send> WriteExt for T {
    async fn write_cstring(&mut self, s: &[u8]) -> Result<()> {
        self.write_all(s).await?;
        self.write_all(b"\0").await?;
        Ok(())
    }

    async fn write_compressed_int(&mut self, x: u32) -> Result<usize> {
        let mut temp = x;
        let mut len = 0;

        while temp > 0x7f {
            self.write_all(&[(0x80 | temp & 0x7f) as u8]).await?;
            len += 1;
            temp &= !0x7f;
            temp >>= 7;
        }

        self.write_all(&[temp as u8]).await?;
        Ok(len + 1)
    }
}

#[must_use]
pub const fn compressed_int_len(x: u32) -> usize {
    let mut temp = x;
    let mut len = 0;

    while temp > 0x7f {
        len += 1;
        temp &= !0x7f;
        temp >>= 7;
    }

    len + 1
}
