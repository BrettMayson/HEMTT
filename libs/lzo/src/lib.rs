mod compress;
mod decompress;
use std::mem;
use std::slice;

use thiserror::Error;

#[must_use]
/// for a given `size` computes the worst case size that the compresed result can be
pub const fn worst_compress(size: usize) -> usize {
    (size) + ((size) / 16) + 64 + 3
}

#[cfg(feature = "compress")]
const LZO1X_1_MEM_COMPRESS: usize = 8192 * 16;
#[cfg(feature = "compress")]
const LZO1X_MEM_COMPRESS: usize = LZO1X_1_MEM_COMPRESS;

#[repr(i32)]
#[derive(Debug, Eq, PartialEq, Error)]
pub enum LzoError {
    #[error("Ok")]
    Ok = 0,
    #[error("Error")]
    Error = -1,
    #[error("OutOfMemory")]
    OutOfMemory = -2,
    #[error("NotCompressible")]
    NotCompressible = -3,
    #[error("InputOverrun")]
    InputOverrun = -4,
    #[error("OutputOverrun")]
    OutputOverrun = -5,
    #[error("LookbehindOverrun")]
    LookbehindOverrun = -6,
    #[error("EOFNotFound")]
    EofNotFound = -7,
    #[error("InputNotConsumed")]
    InputNotConsumed = -8,
    #[error("NotYetImplemented")]
    NotYetImplemented = -9,
    #[error("InvalidArgument")]
    InvalidArgument = -10,
}

#[cfg(feature = "compress")]
/// compress `input` into `output`
/// returns an error if the Vec is not large enough
///
/// # Errors
/// [`LzoError`] if an error occurs
pub fn compress(input: &[u8], output: &mut Vec<u8>) -> Result<(), LzoError> {
    unsafe {
        let wrkmem = libc::malloc(LZO1X_MEM_COMPRESS);
        let mut out_len = output.capacity();
        let err = compress::lzo1x_1_compress(
            input.as_ptr(),
            input.len(),
            output.as_mut_ptr(),
            &mut out_len,
            wrkmem,
        );

        output.set_len(out_len);
        libc::free(wrkmem);
        let res = mem::transmute::<i32, LzoError>(err);
        if res == LzoError::Ok {
            Ok(())
        } else {
            Err(res)
        }
    }
}

#[cfg(feature = "compress")]
/// returns a slice containing the compressed data
///
/// # Errors
/// [`LzoError`] if an error occurs
pub fn compress_to_slice<'a>(in_: &[u8], out: &'a mut [u8]) -> Result<&'a mut [u8], LzoError> {
    unsafe {
        let wrkmem = libc::malloc(LZO1X_MEM_COMPRESS);
        let mut out_len = out.len();
        let err = compress::lzo1x_1_compress(
            in_.as_ptr(),
            in_.len(),
            out.as_mut_ptr(),
            &mut out_len,
            wrkmem,
        );
        libc::free(wrkmem);
        let res = mem::transmute::<i32, LzoError>(err);
        if res == LzoError::Ok {
            Ok(slice::from_raw_parts_mut(out.as_mut_ptr(), out_len))
        } else {
            Err(res)
        }
    }
}

#[cfg(feature = "decompress")]
/// returns a slice containing the decompressed data
///
/// # Errors
/// [`LzoError`] if an error occurs
pub fn decompress_to_slice<'a>(in_: &[u8], out: &'a mut [u8]) -> Result<&'a mut [u8], LzoError> {
    unsafe {
        let mut out_len = out.len();
        let err = decompress::lzo1x_decompress_safe(
            in_.as_ptr(),
            in_.len(),
            out.as_mut_ptr(),
            &mut out_len,
        );
        let res = mem::transmute::<i32, LzoError>(err);
        if res == LzoError::Ok {
            Ok(slice::from_raw_parts_mut(out.as_mut_ptr(), out_len))
        } else {
            Err(res)
        }
    }
}

#[cfg(test)]
#[test]
fn compress_and_back() {
    unsafe {
        let data = [
            0u8, 2, 3, 4, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4,
            2, 2, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4, 2, 2, 4, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4, 2, 2, 4, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4,
        ];
        let dst_len: usize = worst_compress(mem::size_of_val(&data));
        let mut v = Vec::with_capacity(dst_len);
        let dst = libc::malloc(dst_len);
        let dst = slice::from_raw_parts_mut(dst.cast::<u8>(), dst_len);
        let dst = compress_to_slice(&data, dst).unwrap();
        compress(&data, &mut v).unwrap();
        println!("{}", dst.len());

        let dec_dst = libc::malloc(mem::size_of_val(&data));
        let result_len = mem::size_of_val(&data);
        let dec_dst = slice::from_raw_parts_mut(dec_dst.cast::<u8>(), result_len);
        let result = decompress_to_slice(dst, dec_dst).unwrap();
        println!("{}", result.len());
        assert!(result.len() == mem::size_of_val(&data));
        assert!(&data[..] == result);
    }
}
