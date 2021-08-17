#![allow(clippy::nonminimal_bool)]

unsafe extern "C" fn get_unaligned_le16(p: *const ::std::os::raw::c_void) -> u16 {
    let input: *const u8 = p as *const u8;
    (*input.offset(0_isize) as i32 | (*input.offset(1_isize) as i32) << 8_i32) as u16
}

#[no_mangle]
pub unsafe extern "C" fn lzo1x_decompress_safe(
    in_: *const u8,
    in_len: usize,
    out: *mut u8,
    out_len: *mut usize,
) -> i32 {
    let mut current_block;
    let mut op: *mut u8;
    let mut ip: *const u8;
    let mut t: usize;
    let mut next: usize;
    let mut state: usize = 0_usize;
    let mut m_pos: *const u8;
    let ip_end: *const u8 = in_.add(in_len);
    let op_end: *mut u8 = out.add(*out_len);
    op = out;
    ip = in_;
    if in_len >= 3_usize {
        if i32::from(*ip) > 17_i32 {
            t = (*{
                let old = ip;
                ip = ip.offset(1_isize);
                old
            } as i32
                - 17_i32) as usize;
            if t < 4_usize {
                next = t;
                state = next;
                t = next;
                if !(((ip_end as isize).wrapping_sub(ip as isize)
                    / ::std::mem::size_of::<u8>() as isize) as usize
                    >= t.wrapping_add(3_usize))
                {
                    current_block = 64;
                } else if !(((op_end as isize).wrapping_sub(op as isize)
                    / ::std::mem::size_of::<u8>() as isize) as usize
                    >= t)
                {
                    current_block = 63;
                } else {
                    loop {
                        if t == 0_usize {
                            break;
                        }
                        *{
                            let old = op;
                            op = op.offset(1_isize);
                            old
                        } = *{
                            let old = ip;
                            ip = ip.offset(1_isize);
                            old
                        };
                        t = t.wrapping_sub(1_usize);
                    }
                    current_block = 11;
                }
            } else if !(((op_end as isize).wrapping_sub(op as isize)
                / ::std::mem::size_of::<u8>() as isize) as usize
                >= t)
            {
                current_block = 63;
            } else if !(((ip_end as isize).wrapping_sub(ip as isize)
                / ::std::mem::size_of::<u8>() as isize) as usize
                >= t.wrapping_add(3_usize))
            {
                current_block = 64;
            } else {
                loop {
                    *{
                        let old = op;
                        op = op.offset(1_isize);
                        old
                    } = *{
                        let old = ip;
                        ip = ip.offset(1_isize);
                        old
                    };
                    if !({
                        t = t.wrapping_sub(1_usize);
                        t
                    } > 0_usize)
                    {
                        break;
                    }
                }
                state = 4_usize;
                current_block = 11;
            }
        } else {
            current_block = 11;
        }
        if current_block == 64 {
        } else {
            'loop11: loop {
                if current_block == 11 {
                    t = *{
                        let old = ip;
                        ip = ip.offset(1_isize);
                        old
                    } as usize;
                    if t < 16_usize {
                        if state == 0_usize {
                            if t == 0_usize {
                                let mut offset: usize;
                                let ip_last: *const u8 = ip;
                                loop {
                                    if i32::from(*ip) != 0_i32 {
                                        break;
                                    }
                                    ip = ip.offset(1_isize);
                                    if !(((ip_end as isize).wrapping_sub(ip as isize)
                                        / ::std::mem::size_of::<u8>() as isize)
                                        as usize
                                        >= 1_usize)
                                    {
                                        current_block = 64;
                                        break 'loop11;
                                    }
                                }
                                offset = ((ip as isize).wrapping_sub(ip_last as isize)
                                    / ::std::mem::size_of::<u8>() as isize)
                                    as usize;
                                if offset
                                    > (!0_i32 as usize).wrapping_div(255_usize).wrapping_sub(2_usize)
                                {
                                    current_block = 60;
                                    break;
                                }
                                offset = (offset << 8_i32).wrapping_sub(offset);
                                t = t.wrapping_add(offset.wrapping_add(15_usize).wrapping_add(*{
                                    let old = ip;
                                    ip = ip.offset(1_isize);
                                    old
                                }
                                    as usize));
                            }
                            t = t.wrapping_add(3_usize);
                            if !(((op_end as isize).wrapping_sub(op as isize)
                                / ::std::mem::size_of::<u8>() as isize)
                                as usize
                                >= t)
                            {
                                current_block = 63;
                                continue;
                            }
                            if !(((ip_end as isize).wrapping_sub(ip as isize)
                                / ::std::mem::size_of::<u8>() as isize)
                                as usize
                                >= t.wrapping_add(3_usize))
                            {
                                current_block = 64;
                                break;
                            }
                            loop {
                                *{
                                    let old = op;
                                    op = op.offset(1_isize);
                                    old
                                } = *{
                                    let old = ip;
                                    ip = ip.offset(1_isize);
                                    old
                                };
                                if !({
                                    t = t.wrapping_sub(1_usize);
                                    t
                                } > 0_usize)
                                {
                                    break;
                                }
                            }
                            state = 4_usize;
                            current_block = 11;
                            continue;
                        } else if state == 4_usize {
                            next = t & 3_usize;
                            m_pos = op.offset(-((1_i32 + 0x800_i32) as isize)) as *const u8;
                            m_pos = m_pos.offset(-((t >> 2_i32) as isize));
                            m_pos = m_pos.offset(
                                -((i32::from(*{
                                    let old = ip;
                                    ip = ip.offset(1_isize);
                                    old
                                }) << 2_i32) as isize),
                            );
                            t = 3_usize;
                            current_block = 36;
                        } else {
                            next = t & 3_usize;
                            m_pos = op.offset(-1_isize) as *const u8;
                            m_pos = m_pos.offset(-((t >> 2_i32) as isize));
                            m_pos = m_pos.offset(
                                -((i32::from(*{
                                    let old = ip;
                                    ip = ip.offset(1_isize);
                                    old
                                }) << 2_i32) as isize),
                            );
                            if m_pos < out as *const u8 {
                                current_block = 48;
                                break;
                            }
                            if !(((op_end as isize).wrapping_sub(op as isize)
                                / ::std::mem::size_of::<u8>() as isize)
                                as usize
                                >= 2_usize)
                            {
                                current_block = 63;
                                continue;
                            }
                            *op.offset(0_isize) = *m_pos.offset(0_isize);
                            *op.offset(1_isize) = *m_pos.offset(1_isize);
                            op = op.offset(2_isize);
                            current_block = 44;
                        }
                    } else {
                        if t >= 64_usize {
                            next = t & 3_usize;
                            m_pos = op.offset(-1_isize) as *const u8;
                            m_pos = m_pos.offset(-((t >> 2_i32 & 7_usize) as isize));
                            m_pos = m_pos.offset(
                                -((i32::from(*{
                                    let old = ip;
                                    ip = ip.offset(1_isize);
                                    old
                                }) << 3_i32) as isize),
                            );
                            t = (t >> 5_i32)
                                .wrapping_sub(1_usize)
                                .wrapping_add((3_i32 - 1_i32) as usize);
                        } else if t >= 32_usize {
                            t = (t & 31_usize).wrapping_add((3_i32 - 1_i32) as usize);
                            if t == 2_usize {
                                let mut offset: usize;
                                let ip_last: *const u8 = ip;
                                loop {
                                    if i32::from(*ip) != 0_i32 {
                                        break;
                                    }
                                    ip = ip.offset(1_isize);
                                    if !(((ip_end as isize).wrapping_sub(ip as isize)
                                        / ::std::mem::size_of::<u8>() as isize)
                                        as usize
                                        >= 1_usize)
                                    {
                                        current_block = 64;
                                        break 'loop11;
                                    }
                                }
                                offset = ((ip as isize).wrapping_sub(ip_last as isize)
                                    / ::std::mem::size_of::<u8>() as isize)
                                    as usize;
                                if offset
                                    > (!0_i32 as usize).wrapping_div(255_usize).wrapping_sub(2_usize)
                                {
                                    current_block = 30;
                                    break;
                                }
                                offset = (offset << 8_i32).wrapping_sub(offset);
                                t = t.wrapping_add(offset.wrapping_add(31_usize).wrapping_add(*{
                                    let old = ip;
                                    ip = ip.offset(1_isize);
                                    old
                                }
                                    as usize));
                                if !(((ip_end as isize).wrapping_sub(ip as isize)
                                    / ::std::mem::size_of::<u8>() as isize)
                                    as usize
                                    >= 2_usize)
                                {
                                    current_block = 64;
                                    break;
                                }
                            }
                            m_pos = op.offset(-1_isize) as *const u8;
                            next = get_unaligned_le16(ip as *const ::std::os::raw::c_void) as usize;
                            ip = ip.offset(2_isize);
                            m_pos = m_pos.offset(-((next >> 2_i32) as isize));
                            next &= 3_usize;
                        } else {
                            m_pos = op as *const u8;
                            m_pos = m_pos.offset(-(((t & 8_usize) << 11_i32) as isize));
                            t = (t & 7_usize).wrapping_add((3_i32 - 1_i32) as usize);
                            if t == 2_usize {
                                let mut offset: usize;
                                let ip_last: *const u8 = ip;
                                loop {
                                    if *ip as i32 != 0_i32 {
                                        break;
                                    }
                                    ip = ip.offset(1_isize);
                                    if !(((ip_end as isize).wrapping_sub(ip as isize)
                                        / ::std::mem::size_of::<u8>() as isize)
                                        as usize
                                        >= 1_usize)
                                    {
                                        current_block = 64;
                                        break 'loop11;
                                    }
                                }
                                offset = ((ip as isize).wrapping_sub(ip_last as isize)
                                    / ::std::mem::size_of::<u8>() as isize)
                                    as usize;
                                if offset
                                    > (!0_i32 as usize).wrapping_div(255_usize).wrapping_sub(2_usize)
                                {
                                    current_block = 22;
                                    break;
                                }
                                offset = (offset << 8_i32).wrapping_sub(offset);
                                t = t.wrapping_add(offset.wrapping_add(7_usize).wrapping_add(*{
                                    let old = ip;
                                    ip = ip.offset(1_isize);
                                    old
                                }
                                    as usize));
                                if !(((ip_end as isize).wrapping_sub(ip as isize)
                                    / ::std::mem::size_of::<u8>() as isize)
                                    as usize
                                    >= 2_usize)
                                {
                                    current_block = 64;
                                    break;
                                }
                            }
                            next = get_unaligned_le16(ip as *const ::std::os::raw::c_void) as usize;
                            ip = ip.offset(2_isize);
                            m_pos = m_pos.offset(-((next >> 2_i32) as isize));
                            next &= 3_usize;
                            if m_pos == op as *const u8 {
                                current_block = 21;
                                break;
                            }
                            m_pos = m_pos.offset(-0x4000_isize);
                        }
                        current_block = 36;
                    }
                    if current_block == 36 {
                        if m_pos < out as *const u8 {
                            current_block = 48;
                            break;
                        }
                        let oe: *mut u8 = op.add(t);
                        if !(((op_end as isize).wrapping_sub(op as isize)
                            / ::std::mem::size_of::<u8>() as isize)
                            as usize
                            >= t)
                        {
                            current_block = 63;
                            continue;
                        }
                        *op.offset(0_isize) = *m_pos.offset(0_isize);
                        *op.offset(1_isize) = *m_pos.offset(1_isize);
                        op = op.offset(2_isize);
                        m_pos = m_pos.offset(2_isize);
                        loop {
                            *{
                                let old = op;
                                op = op.offset(1_isize);
                                old
                            } = *{
                                let old = m_pos;
                                m_pos = m_pos.offset(1_isize);
                                old
                            };
                            if op >= oe {
                                break;
                            }
                        }
                    }
                    state = next;
                    t = next;
                    if !(((ip_end as isize).wrapping_sub(ip as isize)
                        / ::std::mem::size_of::<u8>() as isize) as usize
                        >= t.wrapping_add(3_usize))
                    {
                        current_block = 64;
                        break;
                    }
                    if !(((op_end as isize).wrapping_sub(op as isize)
                        / ::std::mem::size_of::<u8>() as isize) as usize
                        >= t)
                    {
                        current_block = 63;
                        continue;
                    }
                    loop {
                        if t == 0_usize {
                            current_block = 11;
                            break;
                        }
                        *{
                            let old = op;
                            op = op.offset(1_isize);
                            old
                        } = *{
                            let old = ip;
                            ip = ip.offset(1_isize);
                            old
                        };
                        t = t.wrapping_sub(1_usize);
                    }
                } else {
                    *out_len = ((op as isize).wrapping_sub(out as isize)
                        / ::std::mem::size_of::<u8>() as isize)
                        as usize;
                    return -5_i32;
                }
            }
            if current_block == 64 {
            } else if current_block == 21 {
                *out_len = ((op as isize).wrapping_sub(out as isize)
                    / ::std::mem::size_of::<u8>() as isize) as usize;
                return if t != 3_usize {
                    -1_i32
                } else if ip == ip_end {
                    0_i32
                } else if ip < ip_end {
                    -8_i32
                } else {
                    -4_i32
                };
            } else if current_block == 22 || current_block == 30 {
                return -1_i32;
            } else if current_block == 48 {
                *out_len = ((op as isize).wrapping_sub(out as isize)
                    / ::std::mem::size_of::<u8>() as isize) as usize;
                return -6_i32;
            } else {
                return -1_i32;
            }
        }
    }
    *out_len =
        ((op as isize).wrapping_sub(out as isize) / ::std::mem::size_of::<u8>() as isize) as usize;
    -4_i32
}
