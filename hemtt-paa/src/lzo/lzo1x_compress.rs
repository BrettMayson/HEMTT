extern "C" {
    fn memcpy(
        __dest: *mut ::std::os::raw::c_void,
        __src: *const ::std::os::raw::c_void,
        __n: usize,
    ) -> *mut ::std::os::raw::c_void;
    fn memset(
        __s: *mut ::std::os::raw::c_void,
        __c: i32,
        __n: usize,
    ) -> *mut ::std::os::raw::c_void;
}

unsafe extern "C" fn get_unaligned_le32(p: *const ::std::os::raw::c_void) -> u32 {
    let input: *const u8 = p as *const u8;
    (*input.offset(0_isize) as i32
        | (*input.offset(1_isize) as i32) << 8_i32
        | (*input.offset(2_isize) as i32) << 16_i32
        | (*input.offset(3_isize) as i32) << 24_i32) as u32
}

unsafe extern "C" fn put_unaligned(mut v: u32, p: *mut ::std::os::raw::c_void) {
    memcpy(
        p,
        &mut v as *mut u32 as *const ::std::os::raw::c_void,
        ::std::mem::size_of::<u32>(),
    );
}

unsafe extern "C" fn get_unaligned(p: *const ::std::os::raw::c_void) -> u32 {
    let mut ret: u32 = 0_u32;
    memcpy(
        &mut ret as *mut u32 as *mut ::std::os::raw::c_void,
        p,
        ::std::mem::size_of::<u32>(),
    );
    ret
}

unsafe extern "C" fn lzo1x_1_do_compress(
    in_: *const u8,
    in_len: usize,
    out: *mut u8,
    out_len: *mut usize,
    mut ti: usize,
    wrkmem: *mut ::std::os::raw::c_void,
) -> usize {
    let mut current_block;
    let mut ip: *const u8;
    let mut op: *mut u8;
    let in_end: *const u8 = in_.add(in_len);
    let ip_end: *const u8 = in_.add(in_len).offset(-20_isize);
    let mut ii: *const u8;
    let dict: *mut u16 = wrkmem as *mut u16;
    op = out;
    ip = in_;
    ii = ip;
    ip = ip.add(if ti < 4_usize {
        4_usize.wrapping_sub(ti)
    } else {
        0_usize
    });
    let mut m_pos: *const u8;
    let mut t: usize;
    let mut m_len: usize;
    let mut m_off: usize;
    let mut dv: u32;
    'loop2: loop {
        ip = ip.offset(
            1_isize
                + (((ip as isize).wrapping_sub(ii as isize)
                    / ::std::mem::size_of::<u8>() as isize)
                    >> 5_i32),
        );
        loop {
            if ip >= ip_end {
                break 'loop2;
            }
            dv = get_unaligned_le32(ip as *const ::std::os::raw::c_void);
            t = (dv.wrapping_mul(0x1824_429d_u32) >> (32_i32 - 13_i32)
                & (1_u32 << 13_i32).wrapping_sub(1_u32)) as usize;
            m_pos = in_.offset(*dict.add(t) as isize);
            *dict.add(t) = ((ip as isize).wrapping_sub(in_ as isize)
                / ::std::mem::size_of::<u8>() as isize) as u16;
            if dv != get_unaligned_le32(m_pos as *const ::std::os::raw::c_void) {
                break;
            }
            ii = ii.offset(-(ti as isize));
            ti = 0_usize;
            t = ((ip as isize).wrapping_sub(ii as isize) / ::std::mem::size_of::<u8>() as isize)
                as usize;
            if t != 0_usize {
                if t <= 3_usize {
                    {
                        let _rhs = t;
                        let _lhs = &mut *op.offset(-2_isize);
                        *_lhs = (*_lhs as usize | _rhs) as u8;
                    }
                    put_unaligned(
                        get_unaligned(ii as *const u32 as *const ::std::os::raw::c_void),
                        op as *mut u32 as *mut ::std::os::raw::c_void,
                    );
                    op = op.add(t);
                } else if t <= 16_usize {
                    *{
                        let _old = op;
                        op = op.offset(1_isize);
                        _old
                    } = t.wrapping_sub(3_usize) as u8;
                    put_unaligned(
                        get_unaligned(ii as *const u32 as *const ::std::os::raw::c_void),
                        op as *mut u32 as *mut ::std::os::raw::c_void,
                    );
                    put_unaligned(
                        get_unaligned(
                            ii.offset(4_isize) as *const u32 as *const ::std::os::raw::c_void
                        ),
                        op.offset(4_isize) as *mut u32 as *mut ::std::os::raw::c_void,
                    );
                    put_unaligned(
                        get_unaligned(
                            ii.offset(8_isize) as *const u32 as *const ::std::os::raw::c_void
                        ),
                        op.offset(8_isize) as *mut u32 as *mut ::std::os::raw::c_void,
                    );
                    put_unaligned(
                        get_unaligned(ii.offset(8_isize).offset(4_isize) as *const u32
                            as *const ::std::os::raw::c_void),
                        op.offset(8_isize).offset(4_isize) as *mut u32
                            as *mut ::std::os::raw::c_void,
                    );
                    op = op.add(t);
                } else {
                    if t <= 18_usize {
                        *{
                            let _old = op;
                            op = op.offset(1_isize);
                            _old
                        } = t.wrapping_sub(3_usize) as u8;
                    } else {
                        let mut tt: usize = t.wrapping_sub(18_usize);
                        *{
                            let _old = op;
                            op = op.offset(1_isize);
                            _old
                        } = 0_u8;
                        loop {
                            if tt <= 255_usize {
                                break;
                            }
                            tt = tt.wrapping_sub(255_usize);
                            *{
                                let _old = op;
                                op = op.offset(1_isize);
                                _old
                            } = 0_u8;
                        }
                        *{
                            let _old = op;
                            op = op.offset(1_isize);
                            _old
                        } = tt as u8;
                    }
                    loop {
                        put_unaligned(
                            get_unaligned(ii as *const u32 as *const ::std::os::raw::c_void),
                            op as *mut u32 as *mut ::std::os::raw::c_void,
                        );
                        put_unaligned(
                            get_unaligned(
                                ii.offset(4_isize) as *const u32 as *const ::std::os::raw::c_void
                            ),
                            op.offset(4_isize) as *mut u32 as *mut ::std::os::raw::c_void,
                        );
                        put_unaligned(
                            get_unaligned(
                                ii.offset(8_isize) as *const u32 as *const ::std::os::raw::c_void
                            ),
                            op.offset(8_isize) as *mut u32 as *mut ::std::os::raw::c_void,
                        );
                        put_unaligned(
                            get_unaligned(ii.offset(8_isize).offset(4_isize) as *const u32
                                as *const ::std::os::raw::c_void),
                            op.offset(8_isize).offset(4_isize) as *mut u32
                                as *mut ::std::os::raw::c_void,
                        );
                        op = op.offset(16_isize);
                        ii = ii.offset(16_isize);
                        t = t.wrapping_sub(16_usize);
                        if t < 16_usize {
                            break;
                        }
                    }
                    if t > 0_usize {
                        loop {
                            *{
                                let _old = op;
                                op = op.offset(1_isize);
                                _old
                            } = *{
                                let _old = ii;
                                ii = ii.offset(1_isize);
                                _old
                            };
                            if {
                                t = t.wrapping_sub(1_usize);
                                t
                            } == 0_usize
                            {
                                break;
                            }
                        }
                    }
                }
            }
            m_len = 4_usize;
            if *ip.add(m_len) as i32 == *m_pos.add(m_len) as i32 {
                current_block = 22;
            } else {
                current_block = 31;
            }
            loop {
                if current_block == 22 {
                    m_len = m_len.wrapping_add(1_usize);
                    if *ip.add(m_len) as i32 != *m_pos.add(m_len) as i32 {
                        current_block = 31;
                        continue;
                    }
                    m_len = m_len.wrapping_add(1_usize);
                    if *ip.add(m_len) as i32 != *m_pos.add(m_len) as i32 {
                        current_block = 31;
                        continue;
                    }
                    m_len = m_len.wrapping_add(1_usize);
                    if *ip.add(m_len) as i32 != *m_pos.add(m_len) as i32 {
                        current_block = 31;
                        continue;
                    }
                    m_len = m_len.wrapping_add(1_usize);
                    if *ip.add(m_len) as i32 != *m_pos.add(m_len) as i32 {
                        current_block = 31;
                        continue;
                    }
                    m_len = m_len.wrapping_add(1_usize);
                    if *ip.add(m_len) as i32 != *m_pos.add(m_len) as i32 {
                        current_block = 31;
                        continue;
                    }
                    m_len = m_len.wrapping_add(1_usize);
                    if *ip.add(m_len) as i32 != *m_pos.add(m_len) as i32 {
                        current_block = 31;
                        continue;
                    }
                    m_len = m_len.wrapping_add(1_usize);
                    if *ip.add(m_len) as i32 != *m_pos.add(m_len) as i32 {
                        current_block = 31;
                        continue;
                    }
                    m_len = m_len.wrapping_add(1_usize);
                    if ip.add(m_len) >= ip_end {
                        current_block = 31;
                        continue;
                    }
                    if *ip.add(m_len) as i32 == *m_pos.add(m_len) as i32 {
                        current_block = 22;
                    } else {
                        current_block = 31;
                    }
                } else {
                    m_off = ((ip as isize).wrapping_sub(m_pos as isize)
                        / ::std::mem::size_of::<u8>() as isize)
                        as usize;
                    ip = ip.add(m_len);
                    ii = ip;
                    if m_len <= 8_usize && (m_off <= 0x800_usize) {
                        current_block = 47;
                    } else {
                        current_block = 32;
                    }
                    break;
                }
            }
            if current_block == 32 {
                if m_off <= 0x4000_usize {
                    m_off = m_off.wrapping_sub(1_usize);
                    if m_len <= 33_usize {
                        *{
                            let _old = op;
                            op = op.offset(1_isize);
                            _old
                        } = (32_usize | m_len.wrapping_sub(2_usize)) as u8;
                    } else {
                        m_len = m_len.wrapping_sub(33_usize);
                        *{
                            let _old = op;
                            op = op.offset(1_isize);
                            _old
                        } = 32_u8;
                        loop {
                            if m_len <= 255_usize {
                                break;
                            }
                            m_len = m_len.wrapping_sub(255_usize);
                            *{
                                let _old = op;
                                op = op.offset(1_isize);
                                _old
                            } = 0_u8;
                        }
                        *{
                            let _old = op;
                            op = op.offset(1_isize);
                            _old
                        } = m_len as u8;
                    }
                    *{
                        let _old = op;
                        op = op.offset(1_isize);
                        _old
                    } = (m_off << 2_i32) as u8;
                    *{
                        let _old = op;
                        op = op.offset(1_isize);
                        _old
                    } = (m_off >> 6_i32) as u8;
                } else {
                    m_off = m_off.wrapping_sub(0x4000_usize);
                    if m_len <= 9_usize {
                        *{
                            let _old = op;
                            op = op.offset(1_isize);
                            _old
                        } = (16_usize | m_off >> 11_i32 & 8_usize | m_len.wrapping_sub(2_usize))
                            as u8;
                    } else {
                        m_len = m_len.wrapping_sub(9_usize);
                        *{
                            let _old = op;
                            op = op.offset(1_isize);
                            _old
                        } = (16_usize | m_off >> 11_i32 & 8_usize) as u8;
                        loop {
                            if m_len <= 255_usize {
                                break;
                            }
                            m_len = m_len.wrapping_sub(255_usize);
                            *{
                                let _old = op;
                                op = op.offset(1_isize);
                                _old
                            } = 0_u8;
                        }
                        *{
                            let _old = op;
                            op = op.offset(1_isize);
                            _old
                        } = m_len as u8;
                    }
                }
                *{
                    let _old = op;
                    op = op.offset(1_isize);
                    _old
                } = (m_off << 2_i32) as u8;
                *{
                    let _old = op;
                    op = op.offset(1_isize);
                    _old
                } = (m_off >> 6_i32) as u8;
            } else {
                m_off = m_off.wrapping_sub(1_usize);
                *{
                    let _old = op;
                    op = op.offset(1_isize);
                    _old
                } = (m_len.wrapping_sub(1_usize) << 5_i32 | (m_off & 7_usize) << 2_i32) as u8;
                *{
                    let _old = op;
                    op = op.offset(1_isize);
                    _old
                } = (m_off >> 3_i32) as u8;
            }
        }
    }
    *out_len =
        ((op as isize).wrapping_sub(out as isize) / ::std::mem::size_of::<u8>() as isize) as usize;
    ((in_end as isize).wrapping_sub(ii.offset(-(ti as isize)) as isize)
        / ::std::mem::size_of::<u8>() as isize) as usize
}

#[no_mangle]
pub unsafe extern "C" fn lzo1x_1_compress(
    in_: *const u8,
    in_len: usize,
    out: *mut u8,
    out_len: *mut usize,
    wrkmem: *mut ::std::os::raw::c_void,
) -> i32 {
    let mut current_block;
    let mut ip: *const u8 = in_;
    let mut op: *mut u8 = out;
    let mut l: usize = in_len;
    let mut t: usize = 0_usize;
    loop {
        if l <= 20_usize {
            break;
        }
        let ll: usize = if l <= (0xbfff_i32 + 1_i32) as usize {
            l
        } else {
            (0xbfff_i32 + 1_i32) as usize
        };
        let ll_end: usize = (ip as usize).wrapping_add(ll);
        if ll_end.wrapping_add(t.wrapping_add(ll) >> 5_i32) <= ll_end {
            break;
        }
        memset(
            wrkmem,
            0_i32,
            ((1_u32 << 13_i32) as usize).wrapping_mul(::std::mem::size_of::<u16>()),
        );
        t = lzo1x_1_do_compress(ip, ll, op, out_len, t, wrkmem);
        ip = ip.add(ll);
        op = op.add(*out_len);
        l = l.wrapping_sub(ll);
    }
    t = t.wrapping_add(l);
    if t > 0_usize {
        let mut ii: *const u8 = in_.add(in_len).offset(-(t as isize));
        if op == out && (t <= 238_usize) {
            *{
                let _old = op;
                op = op.offset(1_isize);
                _old
            } = 17_usize.wrapping_add(t) as u8;
        } else if t <= 3_usize {
            let _rhs = t;
            let _lhs = &mut *op.offset(-2_isize);
            *_lhs = (*_lhs as usize | _rhs) as u8;
        } else if t <= 18_usize {
            *{
                let _old = op;
                op = op.offset(1_isize);
                _old
            } = t.wrapping_sub(3_usize) as u8;
        } else {
            let mut tt: usize = t.wrapping_sub(18_usize);
            *{
                let _old = op;
                op = op.offset(1_isize);
                _old
            } = 0_u8;
            loop {
                if tt <= 255_usize {
                    break;
                }
                tt = tt.wrapping_sub(255_usize);
                *{
                    let _old = op;
                    op = op.offset(1_isize);
                    _old
                } = 0_u8;
            }
            *{
                let _old = op;
                op = op.offset(1_isize);
                _old
            } = tt as u8;
        }
        if t >= 16_usize {
            current_block = 16;
        } else {
            current_block = 18;
        }
        loop {
            if current_block == 16 {
                put_unaligned(
                    get_unaligned(ii as *const u32 as *const ::std::os::raw::c_void),
                    op as *mut u32 as *mut ::std::os::raw::c_void,
                );
                put_unaligned(
                    get_unaligned(ii.offset(4_isize) as *const u32 as *const ::std::os::raw::c_void),
                    op.offset(4_isize) as *mut u32 as *mut ::std::os::raw::c_void,
                );
                put_unaligned(
                    get_unaligned(ii.offset(8_isize) as *const u32 as *const ::std::os::raw::c_void),
                    op.offset(8_isize) as *mut u32 as *mut ::std::os::raw::c_void,
                );
                put_unaligned(
                    get_unaligned(ii.offset(8_isize).offset(4_isize) as *const u32
                        as *const ::std::os::raw::c_void),
                    op.offset(8_isize).offset(4_isize) as *mut u32 as *mut ::std::os::raw::c_void,
                );
                op = op.offset(16_isize);
                ii = ii.offset(16_isize);
                t = t.wrapping_sub(16_usize);
                if t >= 16_usize {
                    current_block = 16;
                } else {
                    current_block = 18;
                }
            } else if t > 0_usize {
                current_block = 19;
                break;
            } else {
                current_block = 21;
                break;
            }
        }
        if current_block == 21 {
        } else {
            loop {
                *{
                    let _old = op;
                    op = op.offset(1_isize);
                    _old
                } = *{
                    let _old = ii;
                    ii = ii.offset(1_isize);
                    _old
                };
                if {
                    t = t.wrapping_sub(1_usize);
                    t
                } == 0_usize
                {
                    break;
                }
            }
        }
    }
    *{
        let _old = op;
        op = op.offset(1_isize);
        _old
    } = (16_i32 | 1_i32) as u8;
    *{
        let _old = op;
        op = op.offset(1_isize);
        _old
    } = 0_u8;
    *{
        let _old = op;
        op = op.offset(1_isize);
        _old
    } = 0_u8;
    *out_len =
        ((op as isize).wrapping_sub(out as isize) / ::std::mem::size_of::<u8>() as isize) as usize;
    0_i32
}
