#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_ptr_alignment)]

unsafe extern "C" {
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

const unsafe extern "C" fn get_unaligned_le32(p: *const ::std::os::raw::c_void) -> u32 {
    let input: *const u8 = p.cast::<u8>();
    unsafe {
        (*input.offset(0isize) as i32
            | ((*input.offset(1isize) as i32) << 8i32)
            | ((*input.offset(2isize) as i32) << 16i32)
            | ((*input.offset(3isize) as i32) << 24i32)) as u32
    }
}

unsafe extern "C" fn put_unaligned(mut v: u32, p: *mut ::std::os::raw::c_void) {
    unsafe {
        memcpy(
            p,
            std::ptr::addr_of_mut!(v) as *const ::std::os::raw::c_void,
            ::std::mem::size_of::<u32>(),
        );
    }
}

unsafe extern "C" fn get_unaligned(p: *const ::std::os::raw::c_void) -> u32 {
    let mut ret: u32 = 0u32;
    unsafe {
        memcpy(
            std::ptr::addr_of_mut!(ret).cast::<::std::os::raw::c_void>(),
            p,
            ::std::mem::size_of::<u32>(),
        );
    }
    ret
}

#[allow(clippy::too_many_lines)]
#[allow(clippy::similar_names)]
unsafe extern "C" fn lzo1x_1_do_compress(
    in_: *const u8,
    in_len: usize,
    out: *mut u8,
    out_len: *mut usize,
    mut ti: usize,
    wrkmem: *mut ::std::os::raw::c_void,
) -> usize {
    unsafe {
        let mut current_block;
        let mut ip: *const u8;
        let mut op: *mut u8;
        let in_end: *const u8 = in_.add(in_len);
        let ip_end: *const u8 = in_.add(in_len).offset(-20isize);
        let mut ii: *const u8;
        let dict: *mut u16 = wrkmem.cast::<u16>();
        op = out;
        ip = in_;
        ii = ip;
        ip = ip.add(if ti < 4usize {
            4usize.wrapping_sub(ti)
        } else {
            0usize
        });
        let mut m_pos: *const u8;
        let mut t: usize;
        let mut m_len: usize;
        let mut m_off: usize;
        let mut dv: u32;
        'loop2: loop {
            ip = ip.offset(
                1isize
                    + (((ip as isize).wrapping_sub(ii as isize)
                        / ::std::mem::size_of::<u8>() as isize)
                        >> 5i32),
            );
            loop {
                if ip >= ip_end {
                    break 'loop2;
                }
                dv = get_unaligned_le32(ip.cast::<::std::os::raw::c_void>());
                t = ((dv.wrapping_mul(0x1824_429d_u32) >> (32i32 - 13i32))
                    & (1u32 << 13i32).wrapping_sub(1u32)) as usize;
                m_pos = in_.offset(*dict.add(t) as isize);
                *dict.add(t) = ((ip as isize).wrapping_sub(in_ as isize)
                    / ::std::mem::size_of::<u8>() as isize) as u16;
                if dv != get_unaligned_le32(m_pos.cast::<::std::os::raw::c_void>()) {
                    break;
                }
                ii = ii.offset(-(ti as isize));
                ti = 0usize;
                t = ((ip as isize).wrapping_sub(ii as isize) / ::std::mem::size_of::<u8>() as isize)
                    as usize;
                if t != 0usize {
                    if t <= 3usize {
                        {
                            let rhs = t;
                            let lhs = &mut *op.offset(-2isize);
                            *lhs = (*lhs as usize | rhs) as u8;
                        }
                        put_unaligned(
                            get_unaligned(ii.cast::<u32>().cast::<::std::os::raw::c_void>()),
                            op.cast::<u32>().cast::<::std::os::raw::c_void>(),
                        );
                        op = op.add(t);
                    } else if t <= 16usize {
                        *{
                            let old = op;
                            op = op.offset(1isize);
                            old
                        } = t.wrapping_sub(3usize) as u8;
                        put_unaligned(
                            get_unaligned(ii.cast::<u32>().cast::<::std::os::raw::c_void>()),
                            op.cast::<u32>().cast::<::std::os::raw::c_void>(),
                        );
                        put_unaligned(
                            get_unaligned(
                                ii.offset(4isize)
                                    .cast::<u32>()
                                    .cast::<::std::os::raw::c_void>(),
                            ),
                            op.offset(4isize)
                                .cast::<u32>()
                                .cast::<::std::os::raw::c_void>(),
                        );
                        put_unaligned(
                            get_unaligned(
                                ii.offset(8isize)
                                    .cast::<u32>()
                                    .cast::<::std::os::raw::c_void>(),
                            ),
                            op.offset(8isize)
                                .cast::<u32>()
                                .cast::<::std::os::raw::c_void>(),
                        );
                        put_unaligned(
                            get_unaligned(
                                ii.offset(8isize)
                                    .offset(4isize)
                                    .cast::<u32>()
                                    .cast::<::std::os::raw::c_void>(),
                            ),
                            op.offset(8isize)
                                .offset(4isize)
                                .cast::<u32>()
                                .cast::<::std::os::raw::c_void>(),
                        );
                        op = op.add(t);
                    } else {
                        if t <= 18usize {
                            *{
                                let old = op;
                                op = op.offset(1isize);
                                old
                            } = t.wrapping_sub(3usize) as u8;
                        } else {
                            let mut tt: usize = t.wrapping_sub(18usize);
                            *{
                                let old = op;
                                op = op.offset(1isize);
                                old
                            } = 0u8;
                            loop {
                                if tt <= 255usize {
                                    break;
                                }
                                tt = tt.wrapping_sub(255usize);
                                *{
                                    let old = op;
                                    op = op.offset(1isize);
                                    old
                                } = 0u8;
                            }
                            *{
                                let old = op;
                                op = op.offset(1isize);
                                old
                            } = tt as u8;
                        }
                        loop {
                            put_unaligned(
                                get_unaligned(ii.cast::<u32>().cast::<::std::os::raw::c_void>()),
                                op.cast::<u32>().cast::<::std::os::raw::c_void>(),
                            );
                            put_unaligned(
                                get_unaligned(
                                    ii.offset(4isize)
                                        .cast::<u32>()
                                        .cast::<::std::os::raw::c_void>(),
                                ),
                                op.offset(4isize)
                                    .cast::<u32>()
                                    .cast::<::std::os::raw::c_void>(),
                            );
                            put_unaligned(
                                get_unaligned(
                                    ii.offset(8isize)
                                        .cast::<u32>()
                                        .cast::<::std::os::raw::c_void>(),
                                ),
                                op.offset(8isize)
                                    .cast::<u32>()
                                    .cast::<::std::os::raw::c_void>(),
                            );
                            put_unaligned(
                                get_unaligned(
                                    ii.offset(8isize)
                                        .offset(4isize)
                                        .cast::<u32>()
                                        .cast::<::std::os::raw::c_void>(),
                                ),
                                op.offset(8isize)
                                    .offset(4isize)
                                    .cast::<u32>()
                                    .cast::<::std::os::raw::c_void>(),
                            );
                            op = op.offset(16isize);
                            ii = ii.offset(16isize);
                            t = t.wrapping_sub(16usize);
                            if t < 16usize {
                                break;
                            }
                        }
                        if t > 0usize {
                            loop {
                                *{
                                    let old = op;
                                    op = op.offset(1isize);
                                    old
                                } = *{
                                    let old = ii;
                                    ii = ii.offset(1isize);
                                    old
                                };
                                t -= 1;
                                if t == 0 {
                                    break;
                                }
                            }
                        }
                    }
                }
                m_len = 4usize;
                if i32::from(*ip.add(m_len)) == i32::from(*m_pos.add(m_len)) {
                    current_block = 22;
                } else {
                    current_block = 31;
                }
                loop {
                    if current_block == 22 {
                        m_len = m_len.wrapping_add(1usize);
                        if i32::from(*ip.add(m_len)) != i32::from(*m_pos.add(m_len)) {
                            current_block = 31;
                            continue;
                        }
                        m_len = m_len.wrapping_add(1usize);
                        if i32::from(*ip.add(m_len)) != i32::from(*m_pos.add(m_len)) {
                            current_block = 31;
                            continue;
                        }
                        m_len = m_len.wrapping_add(1usize);
                        if i32::from(*ip.add(m_len)) != i32::from(*m_pos.add(m_len)) {
                            current_block = 31;
                            continue;
                        }
                        m_len = m_len.wrapping_add(1usize);
                        if i32::from(*ip.add(m_len)) != i32::from(*m_pos.add(m_len)) {
                            current_block = 31;
                            continue;
                        }
                        m_len = m_len.wrapping_add(1usize);
                        if i32::from(*ip.add(m_len)) != i32::from(*m_pos.add(m_len)) {
                            current_block = 31;
                            continue;
                        }
                        m_len = m_len.wrapping_add(1usize);
                        if i32::from(*ip.add(m_len)) != i32::from(*m_pos.add(m_len)) {
                            current_block = 31;
                            continue;
                        }
                        m_len = m_len.wrapping_add(1usize);
                        if i32::from(*ip.add(m_len)) != i32::from(*m_pos.add(m_len)) {
                            current_block = 31;
                            continue;
                        }
                        m_len = m_len.wrapping_add(1usize);
                        if ip.add(m_len) >= ip_end {
                            current_block = 31;
                            continue;
                        }
                        if i32::from(*ip.add(m_len)) == i32::from(*m_pos.add(m_len)) {
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
                        if m_len <= 8usize && (m_off <= 0x800usize) {
                            current_block = 47;
                        } else {
                            current_block = 32;
                        }
                        break;
                    }
                }
                if current_block == 32 {
                    if m_off <= 0x4000usize {
                        m_off = m_off.wrapping_sub(1usize);
                        if m_len <= 33usize {
                            *{
                                let old = op;
                                op = op.offset(1isize);
                                old
                            } = (32usize | m_len.wrapping_sub(2usize)) as u8;
                        } else {
                            m_len = m_len.wrapping_sub(33usize);
                            *{
                                let old = op;
                                op = op.offset(1isize);
                                old
                            } = 32i32 as u8;
                            loop {
                                if m_len <= 255usize {
                                    break;
                                }
                                m_len = m_len.wrapping_sub(255usize);
                                *{
                                    let old = op;
                                    op = op.offset(1isize);
                                    old
                                } = 0u8;
                            }
                            *{
                                let old = op;
                                op = op.offset(1isize);
                                old
                            } = m_len as u8;
                        }
                    } else {
                        m_off = m_off.wrapping_sub(0x4000usize);
                        if m_len <= 9usize {
                            *{
                                let old = op;
                                op = op.offset(1isize);
                                old
                            } = (16usize | (m_off >> 11i32) & 8usize | m_len.wrapping_sub(2usize))
                                as u8;
                        } else {
                            m_len = m_len.wrapping_sub(9usize);
                            *{
                                let old = op;
                                op = op.offset(1isize);
                                old
                            } = (16usize | (m_off >> 11i32) & 8usize) as u8;
                            loop {
                                if m_len <= 255usize {
                                    break;
                                }
                                m_len = m_len.wrapping_sub(255usize);
                                *{
                                    let old = op;
                                    op = op.offset(1isize);
                                    old
                                } = 0u8;
                            }
                            *{
                                let old = op;
                                op = op.offset(1isize);
                                old
                            } = m_len as u8;
                        }
                    }
                    *{
                        let old = op;
                        op = op.offset(1isize);
                        old
                    } = (m_off << 2i32) as u8;
                    *{
                        let old = op;
                        op = op.offset(1isize);
                        old
                    } = (m_off >> 6i32) as u8;
                } else {
                    m_off = m_off.wrapping_sub(1usize);
                    *{
                        let old = op;
                        op = op.offset(1isize);
                        old
                    } = ((m_len.wrapping_sub(1usize) << 5i32) | ((m_off & 7usize) << 2i32)) as u8;
                    *{
                        let old = op;
                        op = op.offset(1isize);
                        old
                    } = (m_off >> 3i32) as u8;
                }
            }
        }
        *out_len = ((op as isize).wrapping_sub(out as isize) / ::std::mem::size_of::<u8>() as isize)
            as usize;
        ((in_end as isize).wrapping_sub(ii.offset(-(ti as isize)) as isize)
            / ::std::mem::size_of::<u8>() as isize) as usize
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::too_many_lines)]
#[allow(clippy::similar_names)]
#[allow(clippy::module_name_repetitions)]
pub unsafe extern "C" fn lzo1x_1_compress(
    in_: *const u8,
    in_len: usize,
    out: *mut u8,
    out_len: *mut usize,
    wrkmem: *mut ::std::os::raw::c_void,
) -> i32 {
    unsafe {
        let mut current_block;
        let mut ip: *const u8 = in_;
        let mut op: *mut u8 = out;
        let mut l: usize = in_len;
        let mut t: usize = 0usize;
        loop {
            if l <= 20usize {
                break;
            }
            let ll: usize = if l <= (0xbfffi32 + 1i32) as usize {
                l
            } else {
                (0xbfffi32 + 1i32) as usize
            };
            let ll_end: usize = (ip as usize).wrapping_add(ll);
            if ll_end.wrapping_add(t.wrapping_add(ll) >> 5i32) <= ll_end {
                break;
            }
            memset(
                wrkmem,
                0i32,
                ((1u32 << 13i32) as usize).wrapping_mul(::std::mem::size_of::<u16>()),
            );
            t = lzo1x_1_do_compress(ip, ll, op, out_len, t, wrkmem);
            ip = ip.add(ll);
            op = op.add(*out_len);
            l = l.wrapping_sub(ll);
        }
        t = t.wrapping_add(l);
        if t > 0usize {
            let mut ii: *const u8 = in_.add(in_len).offset(-(t as isize));
            if std::ptr::eq(op, out) && (t <= 238usize) {
                *{
                    let old = op;
                    op = op.offset(1isize);
                    old
                } = 17usize.wrapping_add(t) as u8;
            } else if t <= 3usize {
                let rhs = t;
                let lhs = &mut *op.offset(-2isize);
                *lhs = (*lhs as usize | rhs) as u8;
            } else if t <= 18usize {
                *{
                    let old = op;
                    op = op.offset(1isize);
                    old
                } = t.wrapping_sub(3usize) as u8;
            } else {
                let mut tt: usize = t.wrapping_sub(18usize);
                *{
                    let old = op;
                    op = op.offset(1isize);
                    old
                } = 0u8;
                loop {
                    if tt <= 255usize {
                        break;
                    }
                    tt = tt.wrapping_sub(255usize);
                    *{
                        let old = op;
                        op = op.offset(1isize);
                        old
                    } = 0u8;
                }
                *{
                    let old = op;
                    op = op.offset(1isize);
                    old
                } = tt as u8;
            }
            if t >= 16usize {
                current_block = 16;
            } else {
                current_block = 18;
            }
            loop {
                if current_block == 16 {
                    put_unaligned(
                        get_unaligned(ii.cast::<u32>().cast::<::std::os::raw::c_void>()),
                        op.cast::<u32>().cast::<::std::os::raw::c_void>(),
                    );
                    put_unaligned(
                        get_unaligned(
                            ii.offset(4isize)
                                .cast::<u32>()
                                .cast::<::std::os::raw::c_void>(),
                        ),
                        op.offset(4isize)
                            .cast::<u32>()
                            .cast::<::std::os::raw::c_void>(),
                    );
                    put_unaligned(
                        get_unaligned(
                            ii.offset(8isize)
                                .cast::<u32>()
                                .cast::<::std::os::raw::c_void>(),
                        ),
                        op.offset(8isize)
                            .cast::<u32>()
                            .cast::<::std::os::raw::c_void>(),
                    );
                    put_unaligned(
                        get_unaligned(
                            ii.offset(8isize)
                                .offset(4isize)
                                .cast::<u32>()
                                .cast::<::std::os::raw::c_void>(),
                        ),
                        op.offset(8isize)
                            .offset(4isize)
                            .cast::<u32>()
                            .cast::<::std::os::raw::c_void>(),
                    );
                    op = op.offset(16isize);
                    ii = ii.offset(16isize);
                    t = t.wrapping_sub(16usize);
                    if t >= 16usize {
                        current_block = 16;
                    } else {
                        current_block = 18;
                    }
                } else if t > 0usize {
                    current_block = 19;
                    break;
                } else {
                    current_block = 21;
                    break;
                }
            }
            if current_block != 21 {
                loop {
                    *{
                        let old = op;
                        op = op.offset(1isize);
                        old
                    } = *{
                        let old = ii;
                        ii = ii.offset(1isize);
                        old
                    };
                    t -= 1;
                    if t == 0 {
                        break;
                    }
                }
            }
        }
        *{
            let old = op;
            op = op.offset(1isize);
            old
        } = (16i32 | 1i32) as u8;
        *{
            let old = op;
            op = op.offset(1isize);
            old
        } = 0u8;
        *{
            let old = op;
            op = op.offset(1isize);
            old
        } = 0u8;
        *out_len = ((op as isize).wrapping_sub(out as isize) / ::std::mem::size_of::<u8>() as isize)
            as usize;
        0i32
    }
}
