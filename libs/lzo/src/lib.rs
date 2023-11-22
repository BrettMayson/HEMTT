mod compress;
mod decompress;
use std::mem;
use std::slice;

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
#[derive(Eq, PartialEq)]
pub enum LZOError {
    Ok = 0,
    Error = -1,
    OutOfMemory = -2,
    NotCompressible = -3,
    InputOverrun = -4,
    OutputOverrun = -5,
    LookbehindOverrun = -6,
    EofNotFound = -7,
    InputNotConsumed = -8,
    NotYetImplemented = -9,
    InvalidArgument = -10,
}

#[cfg(feature = "compress")]
/// compress `input` into `output`
/// returns an error if the Vec is not large enough
pub fn compress(input: &[u8], output: &mut Vec<u8>) -> LZOError {
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
        mem::transmute::<i32, LZOError>(err)
    }
}

#[cfg(feature = "compress")]
/// returns a slice containing the compressed data
pub fn compress_to_slice<'a>(in_: &[u8], out: &'a mut [u8]) -> (&'a mut [u8], LZOError) {
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
        (
            slice::from_raw_parts_mut(out.as_mut_ptr(), out_len),
            mem::transmute::<i32, LZOError>(err),
        )
    }
}

#[cfg(feature = "decompress")]
/// returns a slice containing the decompressed data
pub fn decompress_to_slice<'a>(in_: &[u8], out: &'a mut [u8]) -> (&'a mut [u8], LZOError) {
    unsafe {
        let mut out_len = out.len();
        let err = decompress::lzo1x_decompress_safe(
            in_.as_ptr(),
            in_.len(),
            out.as_mut_ptr(),
            &mut out_len,
        );
        (
            slice::from_raw_parts_mut(out.as_mut_ptr(), out_len),
            mem::transmute::<i32, LZOError>(err),
        )
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
        let (dst, err) = compress_to_slice(&data, dst);
        assert!(err == LZOError::Ok);
        let err = compress(&data, &mut v);
        assert!(err == LZOError::Ok);
        println!("{}", dst.len());

        let dec_dst = libc::malloc(mem::size_of_val(&data));
        let result_len = mem::size_of_val(&data);
        let dec_dst = slice::from_raw_parts_mut(dec_dst.cast::<u8>(), result_len);
        let (result, err) = decompress_to_slice(dst, dec_dst);
        assert!(err == LZOError::Ok);
        println!("{}", result.len());
        assert!(result.len() == mem::size_of_val(&data));
        assert!(&data[..] == result);
    }
}
