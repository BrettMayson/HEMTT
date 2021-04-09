mod lzo1x_compress;
mod lzo1x_decompress_safe;
use std::error::Error;
use std::{fmt, mem, slice};

#[allow(dead_code)]
const LZO1X_1_MEM_COMPRESS: usize = 8192 * 16;
#[allow(dead_code)]
const LZO1X_MEM_COMPRESS: usize = LZO1X_1_MEM_COMPRESS;

#[repr(i32)]
#[derive(Debug, PartialEq)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
pub enum LZOError {
    //OK = 0,
    ERROR = -1,
    OUT_OF_MEMORY = -2,
    NOT_COMPRESSIBLE = -3,
    INPUT_OVERRUN = -4,
    OUTPUT_OVERRUN = -5,
    LOOKBEHIND_OVERRUN = -6,
    EOF_NOT_FOUND = -7,
    INPUT_NOT_CONSUMED = -8,
    NOT_YET_IMPLEMENTED = -9,
    INVALID_ARGUMENT = -10,
}

impl fmt::Display for LZOError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LZOError::ERROR => write!(f, "Error"),
            LZOError::OUT_OF_MEMORY => write!(f, "Out of memory"),
            LZOError::NOT_COMPRESSIBLE => write!(f, "Not compressible"),
            LZOError::INPUT_OVERRUN => write!(f, "Input overrun"),
            LZOError::OUTPUT_OVERRUN => write!(f, "Output overrun"),
            LZOError::LOOKBEHIND_OVERRUN => write!(f, "Lookbehind overrun"),
            LZOError::EOF_NOT_FOUND => write!(f, "EOF not found"),
            LZOError::INPUT_NOT_CONSUMED => write!(f, "Input not consumed"),
            LZOError::NOT_YET_IMPLEMENTED => write!(f, "Not yet implemented"),
            LZOError::INVALID_ARGUMENT => write!(f, "Invalid argument"),
        }
    }
}

impl Error for LZOError {}

pub struct LZOContext {
    wrkmem: *mut libc::c_void,
}

impl Drop for LZOContext {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.wrkmem);
        }
    }
}

impl LZOContext {
    pub fn new() -> LZOContext {
        LZOContext {
            wrkmem: unsafe { libc::malloc(LZO1X_MEM_COMPRESS) },
        }
    }

    /// compress `input` into `output`
    /// returns an error if the Vec is not large enough
    pub fn compress(&mut self, input: &[u8], output: &mut Vec<u8>) -> Result<(), LZOError> {
        unsafe {
            let mut out_len = output.capacity();
            let err = lzo1x_compress::lzo1x_1_compress(
                input.as_ptr(),
                input.len(),
                output.as_mut_ptr(),
                &mut out_len,
                &mut *self.wrkmem as *mut _ as *mut _,
            );

            output.set_len(out_len);
            if err == 0 {
                Ok(())
            } else {
                Err(mem::transmute::<i32, LZOError>(err))
            }
        }
    }

    /// returns a slice containing the compressed data
    pub fn compress_to_slice<'a>(
        &mut self,
        in_: &[u8],
        out: &'a mut [u8],
    ) -> Result<&'a mut [u8], LZOError> {
        unsafe {
            let mut out_len = out.len();
            let err = lzo1x_compress::lzo1x_1_compress(
                in_.as_ptr(),
                in_.len(),
                out.as_mut_ptr(),
                &mut out_len,
                &mut *self.wrkmem as *mut _ as *mut _,
            );
            if err == 0 {
                Ok(slice::from_raw_parts_mut(out.as_mut_ptr(), out_len))
            } else {
                Err(mem::transmute::<i32, LZOError>(err))
            }
        }
    }

    /// returns a slice containing the decompressed data
    pub fn decompress_to_slice<'a>(
        in_: &[u8],
        out: &'a mut [u8],
    ) -> Result<&'a mut [u8], LZOError> {
        unsafe {
            let mut out_len = out.len();
            let err = lzo1x_decompress_safe::lzo1x_decompress_safe(
                in_.as_ptr(),
                in_.len(),
                out.as_mut_ptr(),
                &mut out_len,
            );
            if err == 0 {
                Ok(slice::from_raw_parts_mut(out.as_mut_ptr(), out_len))
            } else {
                Err(mem::transmute::<i32, LZOError>(err))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub fn worst_compress(size: usize) -> usize {
        (size) + ((size) / 16) + 64 + 3
    }

    #[test]
    fn it_works() {
        unsafe {
            let data = [
                0u8, 2, 3, 4, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3,
                4, 2, 2, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4, 2, 2, 4,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4, 2, 2, 4, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4,
            ];
            let dst_len: usize = worst_compress(mem::size_of_val(&data));
            let mut v = Vec::with_capacity(dst_len);
            let dst = libc::malloc(dst_len);
            let mut ctx = LZOContext::new();
            let mut dst = slice::from_raw_parts_mut(dst as *mut u8, dst_len);
            let result = ctx.compress_to_slice(&data, &mut dst);
            assert_eq!(result.is_ok(), true);
            let dst = result.unwrap();
            let result = ctx.compress(&data, &mut v);
            assert_eq!(result.is_ok(), true);
            println!("{}", dst.len());

            let dec_dst = libc::malloc(mem::size_of_val(&data));
            let result_len = mem::size_of_val(&data);
            let mut dec_dst = slice::from_raw_parts_mut(dec_dst as *mut u8, result_len);
            let result = LZOContext::decompress_to_slice(&dst, &mut dec_dst);
            assert_eq!(result.is_ok(), true);
            let result = result.unwrap();
            println!("{}", result.len());
            assert_eq!(result.len(), mem::size_of_val(&data));
            assert!(&data[..] == result);
        }
    }
}
