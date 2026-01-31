use std::alloc::{Layout, alloc, dealloc};
use std::mem;
use std::slice;

mod compress;
mod decompress;

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
///
/// # Panics
/// If the layout creation fails
pub fn compress(input: &[u8], output: &mut Vec<u8>) -> Result<(), LzoError> {
    unsafe {
        use std::ffi::c_void;

        let layout = Layout::from_size_align(LZO1X_MEM_COMPRESS, std::mem::align_of::<u8>())
            .expect("Failed to create layout");
        let wrkmem = alloc(layout).cast::<c_void>();
        // let wrkmem = libc::malloc(LZO1X_MEM_COMPRESS);
        let mut out_len = output.capacity();
        let err = compress::lzo1x_1_compress(
            input.as_ptr(),
            input.len(),
            output.as_mut_ptr(),
            &raw mut out_len,
            wrkmem,
        );

        output.set_len(out_len);
        dealloc(wrkmem.cast::<u8>(), layout);
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
///
/// # Panics
/// If the layout creation fails
pub fn compress_to_slice<'a>(in_: &[u8], out: &'a mut [u8]) -> Result<&'a mut [u8], LzoError> {
    unsafe {
        let layout = Layout::from_size_align(LZO1X_MEM_COMPRESS, std::mem::align_of::<u8>())
            .expect("Failed to create layout");
        let wrkmem = alloc(layout).cast::<std::ffi::c_void>();
        let mut out_len = out.len();
        let err = compress::lzo1x_1_compress(
            in_.as_ptr(),
            in_.len(),
            out.as_mut_ptr(),
            &raw mut out_len,
            wrkmem,
        );
        dealloc(wrkmem.cast::<u8>(), layout);
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
            &raw mut out_len,
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
#[allow(clippy::unwrap_used)]
#[test]
fn compress_and_back() {
    unsafe {
        let mut tests = vec![vec![
            0u8, 2, 3, 4, 2, 3, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4,
            2, 2, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4, 2, 2, 4, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4, 2, 2, 4, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 4,
        ]];
        for _ in 0..99 {
            let mut v = Vec::with_capacity(255);
            for _ in 0..=(rand::random::<u32>() % 10_000) {
                v.push(rand::random::<u8>());
            }
            tests.push(v);
        }
        for data in tests {
            let size = mem::size_of_val(&data[0]) * data.len();
            let dst_len: usize = worst_compress(size);
            let mut v = Vec::with_capacity(dst_len);
            let dst = alloc(Layout::from_size_align(dst_len, mem::align_of::<u8>()).unwrap());
            let dst = slice::from_raw_parts_mut(dst.cast::<u8>(), dst_len);
            let dst = compress_to_slice(&data, dst).unwrap();
            compress(&data, &mut v).unwrap();

            let dec_dst = alloc(Layout::from_size_align(size, mem::align_of::<u8>()).unwrap());
            let dec_dst = slice::from_raw_parts_mut(dec_dst.cast::<u8>(), size);
            let result = decompress_to_slice(dst, dec_dst).unwrap();
            assert!(result.len() == size);
            assert!(&data[..] == result);
        }
    }
}
