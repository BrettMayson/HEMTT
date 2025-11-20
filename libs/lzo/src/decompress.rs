#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_ptr_alignment)]

use crate::LzoError;

const unsafe extern "C" fn get_unaligned_le16(p: *const ::std::os::raw::c_void) -> u16 {
    let input: *const u8 = p.cast::<u8>();
    unsafe { (*input.offset(0isize) as i32 | ((*input.offset(1isize) as i32) << 8i32)) as u16 }
}

#[unsafe(no_mangle)]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn lzo1x_decompress_safe(
    in_: *const u8,
    in_len: usize,
    out: *mut u8,
    out_len: *mut usize,
) -> i32 {
    unsafe {
        let mut current_block;
        let mut op: *mut u8;
        let mut ip: *const u8;
        let mut t: usize;
        let mut next: usize;
        let mut state: usize = 0usize;
        let mut m_pos: *const u8;
        let ip_end: *const u8 = in_.add(in_len);
        let op_end: *mut u8 = out.add(*out_len);
        op = out;
        ip = in_;
        if in_len >= 3usize {
            if i32::from(*ip) > 17i32 {
                t = (i32::from(*{
                    let old = ip;
                    ip = ip.offset(1isize);
                    old
                }) - 17i32) as usize;
                if t < 4usize {
                    next = t;
                    state = next;
                    t = next;
                    if (((ip_end as isize).wrapping_sub(ip as isize)
                        / ::std::mem::size_of::<u8>() as isize) as usize)
                        < t.wrapping_add(3usize)
                    {
                        current_block = 64;
                    } else if (((op_end as isize).wrapping_sub(op as isize)
                        / ::std::mem::size_of::<u8>() as isize)
                        as usize)
                        < t
                    {
                        current_block = 63;
                    } else {
                        loop {
                            if t == 0usize {
                                break;
                            }
                            *{
                                let old = op;
                                op = op.offset(1isize);
                                old
                            } = *{
                                let old = ip;
                                ip = ip.offset(1isize);
                                old
                            };
                            t = t.wrapping_sub(1usize);
                        }
                        current_block = 11;
                    }
                } else if (((op_end as isize).wrapping_sub(op as isize)
                    / ::std::mem::size_of::<u8>() as isize) as usize)
                    < t
                {
                    current_block = 63;
                } else if (((ip_end as isize).wrapping_sub(ip as isize)
                    / ::std::mem::size_of::<u8>() as isize) as usize)
                    < t.wrapping_add(3usize)
                {
                    current_block = 64;
                } else {
                    loop {
                        *{
                            let old = op;
                            op = op.offset(1isize);
                            old
                        } = *{
                            let old = ip;
                            ip = ip.offset(1isize);
                            old
                        };
                        t -= 1;
                        if t == 0 {
                            break;
                        }
                    }
                    state = 4usize;
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
                            ip = ip.offset(1isize);
                            old
                        } as usize;
                        if t < 16usize {
                            if state == 0usize {
                                if t == 0usize {
                                    let mut offset: usize;
                                    let ip_last: *const u8 = ip;
                                    loop {
                                        if i32::from(*ip) != 0i32 {
                                            break;
                                        }
                                        ip = ip.offset(1isize);
                                        if (((ip_end as isize).wrapping_sub(ip as isize)
                                            / ::std::mem::size_of::<u8>() as isize)
                                            as usize)
                                            < 1usize
                                        {
                                            current_block = 64;
                                            break 'loop11;
                                        }
                                    }
                                    offset = ((ip as isize).wrapping_sub(ip_last as isize)
                                        / ::std::mem::size_of::<u8>() as isize)
                                        as usize;
                                    if offset
                                        > (!0i32 as usize)
                                            .wrapping_div(255usize)
                                            .wrapping_sub(2usize)
                                    {
                                        current_block = 60;
                                        break;
                                    }
                                    offset = (offset << 8i32).wrapping_sub(offset);
                                    t = t.wrapping_add(offset.wrapping_add(15usize).wrapping_add(
                                        *{
                                            let old = ip;
                                            ip = ip.offset(1isize);
                                            old
                                        } as usize,
                                    ));
                                }
                                t = t.wrapping_add(3usize);
                                if (((op_end as isize).wrapping_sub(op as isize)
                                    / ::std::mem::size_of::<u8>() as isize)
                                    as usize)
                                    < t
                                {
                                    current_block = 63;
                                    continue;
                                }
                                if (((ip_end as isize).wrapping_sub(ip as isize)
                                    / ::std::mem::size_of::<u8>() as isize)
                                    as usize)
                                    < t.wrapping_add(3usize)
                                {
                                    current_block = 64;
                                    break;
                                }
                                loop {
                                    *{
                                        let old = op;
                                        op = op.offset(1isize);
                                        old
                                    } = *{
                                        let old = ip;
                                        ip = ip.offset(1isize);
                                        old
                                    };
                                    t -= 1;
                                    if t == 0 {
                                        break;
                                    }
                                }
                                state = 4usize;
                                current_block = 11;
                                continue;
                            } else if state != 4usize {
                                next = t & 3usize;
                                m_pos = op.offset(-1isize).cast_const();
                                m_pos = m_pos.offset(-((t >> 2i32) as isize));
                                m_pos = m_pos.offset(
                                    -((i32::from(*{
                                        let old = ip;
                                        ip = ip.offset(1isize);
                                        old
                                    }) << 2i32) as isize),
                                );
                                if m_pos < out.cast_const() {
                                    current_block = 48;
                                    break;
                                }
                                if (((op_end as isize).wrapping_sub(op as isize)
                                    / ::std::mem::size_of::<u8>() as isize)
                                    as usize)
                                    < 2usize
                                {
                                    current_block = 63;
                                    continue;
                                }
                                *op.offset(0isize) = *m_pos.offset(0isize);
                                *op.offset(1isize) = *m_pos.offset(1isize);
                                op = op.offset(2isize);
                                current_block = 44;
                            } else {
                                next = t & 3usize;
                                m_pos = op.offset(-((1i32 + 0x800i32) as isize)).cast_const();
                                m_pos = m_pos.offset(-((t >> 2i32) as isize));
                                m_pos = m_pos.offset(
                                    -((i32::from(*{
                                        let old = ip;
                                        ip = ip.offset(1isize);
                                        old
                                    }) << 2i32) as isize),
                                );
                                t = 3usize;
                                current_block = 36;
                            }
                        } else {
                            if t >= 64usize {
                                next = t & 3usize;
                                m_pos = op.offset(-1isize).cast_const();
                                m_pos = m_pos.offset(-(((t >> 2i32) & 7usize) as isize));
                                m_pos = m_pos.offset(
                                    -((i32::from(*{
                                        let old = ip;
                                        ip = ip.offset(1isize);
                                        old
                                    }) << 3i32) as isize),
                                );
                                t = (t >> 5i32)
                                    .wrapping_sub(1usize)
                                    .wrapping_add((3i32 - 1i32) as usize);
                            } else if t >= 32usize {
                                t = (t & 31usize).wrapping_add((3i32 - 1i32) as usize);
                                if t == 2usize {
                                    let mut offset: usize;
                                    let ip_last: *const u8 = ip;
                                    loop {
                                        if i32::from(*ip) != 0i32 {
                                            break;
                                        }
                                        ip = ip.offset(1isize);
                                        if (((ip_end as isize).wrapping_sub(ip as isize)
                                            / ::std::mem::size_of::<u8>() as isize)
                                            as usize)
                                            < 1usize
                                        {
                                            current_block = 64;
                                            break 'loop11;
                                        }
                                    }
                                    offset = ((ip as isize).wrapping_sub(ip_last as isize)
                                        / ::std::mem::size_of::<u8>() as isize)
                                        as usize;
                                    if offset
                                        > (!0i32 as usize)
                                            .wrapping_div(255usize)
                                            .wrapping_sub(2usize)
                                    {
                                        current_block = 30;
                                        break;
                                    }
                                    offset = (offset << 8i32).wrapping_sub(offset);
                                    t = t.wrapping_add(offset.wrapping_add(31usize).wrapping_add(
                                        *{
                                            let old = ip;
                                            ip = ip.offset(1isize);
                                            old
                                        } as usize,
                                    ));
                                    if (((ip_end as isize).wrapping_sub(ip as isize)
                                        / ::std::mem::size_of::<u8>() as isize)
                                        as usize)
                                        < 2usize
                                    {
                                        current_block = 64;
                                        break;
                                    }
                                }
                                m_pos = op.offset(-1isize).cast_const();
                                next = get_unaligned_le16(ip.cast::<::std::os::raw::c_void>())
                                    as usize;
                                ip = ip.offset(2isize);
                                m_pos = m_pos.offset(-((next >> 2i32) as isize));
                                next &= 3usize;
                            } else {
                                m_pos = op.cast_const();
                                m_pos = m_pos.offset(-(((t & 8usize) << 11i32) as isize));
                                t = (t & 7usize).wrapping_add((3i32 - 1i32) as usize);
                                if t == 2usize {
                                    let mut offset: usize;
                                    let ip_last: *const u8 = ip;
                                    loop {
                                        if i32::from(*ip) != 0i32 {
                                            break;
                                        }
                                        ip = ip.offset(1isize);
                                        if (((ip_end as isize).wrapping_sub(ip as isize)
                                            / ::std::mem::size_of::<u8>() as isize)
                                            as usize)
                                            < 1usize
                                        {
                                            current_block = 64;
                                            break 'loop11;
                                        }
                                    }
                                    offset = ((ip as isize).wrapping_sub(ip_last as isize)
                                        / ::std::mem::size_of::<u8>() as isize)
                                        as usize;
                                    if offset
                                        > (!0i32 as usize)
                                            .wrapping_div(255usize)
                                            .wrapping_sub(2usize)
                                    {
                                        current_block = 22;
                                        break;
                                    }
                                    offset = (offset << 8i32).wrapping_sub(offset);
                                    t = t.wrapping_add(offset.wrapping_add(7usize).wrapping_add(
                                        *{
                                            let old = ip;
                                            ip = ip.offset(1isize);
                                            old
                                        } as usize,
                                    ));
                                    if (((ip_end as isize).wrapping_sub(ip as isize)
                                        / ::std::mem::size_of::<u8>() as isize)
                                        as usize)
                                        < 2usize
                                    {
                                        current_block = 64;
                                        break;
                                    }
                                }
                                next = get_unaligned_le16(ip.cast::<::std::os::raw::c_void>())
                                    as usize;
                                ip = ip.offset(2isize);
                                m_pos = m_pos.offset(-((next >> 2i32) as isize));
                                next &= 3usize;
                                if std::ptr::eq(m_pos, op.cast_const()) {
                                    current_block = 21;
                                    break;
                                }
                                m_pos = m_pos.offset(-0x4000isize);
                            }
                            current_block = 36;
                        }
                        if current_block == 36 {
                            if m_pos < out.cast_const() {
                                current_block = 48;
                                break;
                            }
                            let oe: *mut u8 = op.add(t);
                            if (((op_end as isize).wrapping_sub(op as isize)
                                / ::std::mem::size_of::<u8>() as isize)
                                as usize)
                                < t
                            {
                                current_block = 63;
                                continue;
                            }
                            *op.offset(0isize) = *m_pos.offset(0isize);
                            *op.offset(1isize) = *m_pos.offset(1isize);
                            op = op.offset(2isize);
                            m_pos = m_pos.offset(2isize);
                            loop {
                                *{
                                    let old = op;
                                    op = op.offset(1isize);
                                    old
                                } = *{
                                    let old = m_pos;
                                    m_pos = m_pos.offset(1isize);
                                    old
                                };
                                if op >= oe {
                                    break;
                                }
                            }
                        }
                        state = next;
                        t = next;
                        if (((ip_end as isize).wrapping_sub(ip as isize)
                            / ::std::mem::size_of::<u8>() as isize)
                            as usize)
                            < t.wrapping_add(3usize)
                        {
                            current_block = 64;
                            break;
                        }
                        if (((op_end as isize).wrapping_sub(op as isize)
                            / ::std::mem::size_of::<u8>() as isize)
                            as usize)
                            < t
                        {
                            current_block = 63;
                            continue;
                        }
                        loop {
                            if t == 0usize {
                                current_block = 11;
                                break;
                            }
                            *{
                                let old = op;
                                op = op.offset(1isize);
                                old
                            } = *{
                                let old = ip;
                                ip = ip.offset(1isize);
                                old
                            };
                            t = t.wrapping_sub(1usize);
                        }
                    } else {
                        *out_len = ((op as isize).wrapping_sub(out as isize)
                            / ::std::mem::size_of::<u8>() as isize)
                            as usize;
                        return LzoError::OutputOverrun as i32;
                    }
                }
                if current_block == 64 {
                } else if current_block == 21 {
                    *out_len = ((op as isize).wrapping_sub(out as isize)
                        / ::std::mem::size_of::<u8>() as isize)
                        as usize;
                    return if t != 3usize {
                        -1i32
                    } else if std::ptr::eq(ip, ip_end) {
                        0i32
                    } else if ip < ip_end {
                        -8i32
                    } else {
                        -4i32
                    };
                } else if current_block == 22 || current_block == 30 {
                    return -1i32;
                } else if current_block == 48 {
                    *out_len = ((op as isize).wrapping_sub(out as isize)
                        / ::std::mem::size_of::<u8>() as isize)
                        as usize;
                    return -6i32;
                } else {
                    return -1i32;
                }
            }
        }
        *out_len = ((op as isize).wrapping_sub(out as isize) / ::std::mem::size_of::<u8>() as isize)
            as usize;
        -4i32
    }
}
