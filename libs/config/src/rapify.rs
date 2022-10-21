use std::io::Write;

pub trait Rapify {
    fn rapify<O: Write>(&self, output: &mut O, offset: usize) -> Result<usize, std::io::Error>;
    fn rapified_length(&self) -> usize;
    fn rapified_code(&self) -> Option<u8> {
        None
    }
}

pub trait WriteExt: Write {
    fn write_cstring<S: AsRef<[u8]>>(&mut self, s: S) -> std::io::Result<()>;
    fn write_compressed_int(&mut self, x: u32) -> std::io::Result<usize>;
}

impl<T: Write> WriteExt for T {
    fn write_cstring<S: AsRef<[u8]>>(&mut self, s: S) -> std::io::Result<()> {
        self.write_all(s.as_ref())?;
        self.write_all(b"\0")?;
        Ok(())
    }

    fn write_compressed_int(&mut self, x: u32) -> std::io::Result<usize> {
        let mut temp = x;
        let mut len = 0;

        while temp > 0x7f {
            self.write_all(&[(0x80 | temp & 0x7f) as u8])?;
            len += 1;
            temp &= !0x7f;
            temp >>= 7;
        }

        self.write_all(&[temp as u8])?;
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
