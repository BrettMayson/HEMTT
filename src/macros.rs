#[macro_use]

#[macro_export]
macro_rules! iprintln {
    ($e:expr, $($p:ident),*) => {
        println!($e, $($p = $p,)*);
    };
}

#[macro_export]
macro_rules! iprint {
    ($e:expr, $($p:ident),*) => {
        print!($e, $($p = $p,)*);
    };
}

#[macro_export]
macro_rules! ieprintln {
    ($e:expr, $($p:ident),*) => {
        eprintln!($e, $($p = $p,)*);
    };
}

#[macro_export]
macro_rules! ieprint {
    ($e:expr, $($p:ident),*) => {
        eprint!($e, $($p = $p,)*);
    };
}

#[macro_export]
macro_rules! iformat {
    ($e:expr, $($p:ident),*) => {
        format!($e, $($p = $p,)*);
    };
}

#[macro_export]
macro_rules! repeat {
    ($s:expr, $n:expr) => {{
        &std::iter::repeat($s).take($n).collect::<String>()
    }}
}

#[macro_export]
macro_rules! yellow {
    ($s:expr, $m:expr) => {
        crate::niceout!(yellow, $s, $m);
    }
}

#[macro_export]
macro_rules! blue {
    ($s:expr, $m:expr) => {
        crate::niceout!(blue, $s, $m);
    }
}

#[macro_export]
macro_rules! green {
    ($s:expr, $m:expr) => {
        crate::niceout!(green, $s, $m);
    }
}

#[macro_export]
macro_rules! cyan {
    ($s:expr, $m:expr) => {
        crate::niceout!(cyan, $s, $m);
    }
}

#[macro_export]
macro_rules! magenta {
    ($s:expr, $m:expr) => {
        crate::niceout!(magenta, $s, $m);
    }
}

#[macro_export]
macro_rules! red {
    ($s:expr, $m:expr) => {
        crate::niceout!(red, $s, $m);
    }
}

#[macro_export]
macro_rules! black {
    ($s:expr, $m:expr) => {
        crate::niceout!(black, $s, $m);
    }
}

#[macro_export]
macro_rules! white {
    ($s:expr, $m:expr) => {
        crate::niceout!(white, $s, $m);
    }
}

#[macro_export]
macro_rules! nicefmt {
    ($c:ident, $s:expr, $m:expr) => {
        let status = $s.$c().bold();
        let spacer = crate::repeat!(" ", 10 - $s.len());
        let message = $m;
        format!("{spacer}{status} {message}", spacer, status, message)
    }
}

#[macro_export]
macro_rules! niceout {
    ($c:ident, $s:expr, $m:expr) => {
        crate::iprintln!($c, $s, $m);
    }
}

#[macro_export]
macro_rules! finishpb {
    ($pb:ident, $c:ident, $s:expr, $m:expr) => {
        let message =  format!("{}{}", $m, crate::repeat!(" ", 60);
        $pb().finish_print(&crate::nicefmt!($c, $s, message)));
        println!();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_format() {
        let name = "HEMTT";
        assert_eq!("Hello HEMTT", iformat!("Hello {name}", name));
    }

    #[test]
    fn test_repeat() {
        assert_eq!("....", repeat!(".", 4));
    }
}
