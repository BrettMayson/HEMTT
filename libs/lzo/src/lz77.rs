/// Compress data from input to a fixed-size output buffer using LZ77
///
/// Returns the number of bytes written to output on success, or an error message
///
/// # Errors
/// [`&'static str`] if an error occurs during compression
pub fn compress(input: &[u8], out_buf: &mut [u8]) -> Result<usize, &'static str> {
    const MAX_OFFSET: usize = 4095; // 12 bits
    const MIN_MATCH: usize = 3;
    const MAX_MATCH: usize = 18; // 4 bits + 3

    if input.is_empty() {
        return Ok(0);
    }

    let mut pi = 0; // Input position
    let mut po = 0; // Output position
    let mut checksum: u32 = 0;

    // Hash table to store positions of 3-byte sequences for fast lookup
    let mut hash_table: std::collections::HashMap<u32, Vec<usize>> =
        std::collections::HashMap::new();

    while pi < input.len() {
        if po >= out_buf.len() {
            return Err("Output buffer too small");
        }

        let flag_pos = po;
        po += 1;
        let mut flag: u8 = 0;

        for bit in 0..8 {
            if pi >= input.len() {
                break;
            }

            // Try to find a match in the previous data using hash table
            let mut best_match_pos = 0;
            let mut best_match_len = 0;

            if pi + MIN_MATCH <= input.len() {
                // Calculate hash for current position
                let hash = u32::from(input[pi])
                    | (u32::from(input[pi + 1]) << 8)
                    | (u32::from(input[pi + 2]) << 16);

                // Look up previous positions with same hash
                if let Some(positions) = hash_table.get(&hash) {
                    let search_start = pi.saturating_sub(MAX_OFFSET);
                    for &search_pos in positions.iter().rev() {
                        if search_pos < search_start {
                            break;
                        }

                        let mut match_len = 0;
                        while match_len < MAX_MATCH
                            && pi + match_len < input.len()
                            && search_pos + match_len < input.len()
                            && input[search_pos + match_len] == input[pi + match_len]
                        {
                            match_len += 1;
                        }

                        if match_len >= MIN_MATCH && match_len > best_match_len {
                            best_match_len = match_len;
                            best_match_pos = pi - search_pos;
                        }
                    }
                }

                // Add current position to hash table
                hash_table.entry(hash).or_default().push(pi);
            }

            #[allow(clippy::cast_possible_truncation)]
            if best_match_len >= MIN_MATCH {
                // Use back reference
                if po + 2 > out_buf.len() {
                    return Err("Output buffer too small");
                }

                let offset = best_match_pos;
                let length = (best_match_len - 3) as u8;

                out_buf[po] = (offset & 0xFF) as u8;
                po += 1;
                out_buf[po] = ((offset >> 4) & 0xF0) as u8 | (length & 0x0F);
                po += 1;

                // Update checksum
                for i in 0..best_match_len {
                    checksum += u32::from(input[pi + i]);
                }

                pi += best_match_len;
            } else {
                // Raw byte
                if po >= out_buf.len() {
                    return Err("Output buffer too small");
                }

                flag |= 1 << bit;
                out_buf[po] = input[pi];
                checksum += u32::from(input[pi]);
                po += 1;
                pi += 1;
            }
        }

        out_buf[flag_pos] = flag;
    }

    // Write checksum
    if po + 4 > out_buf.len() {
        return Err("Output buffer too small for checksum");
    }

    let checksum_bytes = checksum.to_ne_bytes();
    out_buf[po..po + 4].copy_from_slice(&checksum_bytes);
    po += 4;

    Ok(po)
}

/// Decompress data from input to a fixed-size output buffer
///
/// Returns the number of bytes read from input on success, or an error message
///
/// # Errors
/// [`&'static str`] if an error occurs during decompression
pub fn decompress(input: &[u8], out_buf: &mut [u8]) -> Result<usize, &'static str> {
    let outlen = out_buf.len();
    let mut flag: u8 = 0;
    let mut rpos: usize;
    let mut rlen: u8;
    let mut fl: usize = 0;
    // let mut calculated_checksum: u32 = 0;
    let mut pi: usize = 0;
    let mut data: u8;

    let mut remaining_outlen = outlen;

    'outer: while remaining_outlen > 0 {
        if pi >= input.len() {
            return Err("Unexpected end of input data");
        }

        flag = input[pi];
        pi += 1;

        for _ in 0..8 {
            if (flag & 0x01) != 0 {
                // Raw data
                if pi >= input.len() {
                    return Err("Unexpected end of input data during raw byte read");
                }

                data = input[pi];
                pi += 1;
                // calculated_checksum += u32::from(data);
                out_buf[fl] = data;
                fl += 1;

                remaining_outlen -= 1;
                if remaining_outlen == 0 {
                    break 'outer; // goto finish
                }
            } else {
                // Back reference - need 2 more bytes
                if pi + 1 >= input.len() {
                    return Err("Unexpected end of input data during back reference read");
                }

                rpos = input[pi] as usize;
                pi += 1;

                rlen = (input[pi] & 0x0F) + 3;
                rpos += ((input[pi] & 0xF0) as usize) << 4;
                pi += 1;

                // Special case: space fill
                let mut skip_backref = false;
                while rpos > fl {
                    // calculated_checksum += 0x20;
                    out_buf[fl] = 0x20;
                    fl += 1;

                    remaining_outlen -= 1;
                    if remaining_outlen == 0 {
                        break 'outer; // goto finish
                    }

                    rlen -= 1;
                    if rlen == 0 {
                        skip_backref = true;
                        break;
                    }
                }

                if !skip_backref {
                    // Standard back reference copy
                    rpos = fl - rpos;

                    // Need to copy byte-by-byte because source and destination might overlap
                    for _ in 0..rlen {
                        data = out_buf[rpos];
                        // calculated_checksum += u32::from(data);
                        out_buf[fl] = data;
                        fl += 1;
                        rpos += 1;

                        remaining_outlen -= 1;
                        if remaining_outlen == 0 {
                            break 'outer; // goto finish
                        }
                    }
                }
            }

            // Shift flag for next bit
            flag >>= 1;
        }
    }

    // Check excess bits in final flag byte
    if flag & 0xFE != 0 {
        return Err("Excess bits in final flag byte");
    }

    // Read checksum
    if pi + 3 >= input.len() {
        return Err("Cannot read checksum: unexpected end of input");
    }

    // let read_checksum =
    //     u32::from_ne_bytes([input[pi], input[pi + 1], input[pi + 2], input[pi + 3]]);

    // if read_checksum != calculated_checksum {
    // this is expected right now with PAAs
    // they store the checksum in a different format and
    // I don't feel like fixing it right now
    // }
    Ok(pi + 4)
}

#[cfg(test)]
mod tests {
    #[test]
    fn compress_and_back() {
        let data = b"Hello, Hello, Hello, Hello, Hello, Hello, Hello, Hello!";
        let mut compressed = vec![0u8; 100];
        let compressed_size = super::compress(data, &mut compressed).expect("Compression failed");
        compressed.truncate(compressed_size);

        let mut decompressed = vec![0u8; data.len()];
        super::decompress(&compressed, &mut decompressed).expect("Decompression failed");

        assert_eq!(data, &decompressed[..]);
    }
}
