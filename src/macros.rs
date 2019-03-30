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
    ($s: expr, $n: expr) => {{
        &std::iter::repeat($s).take($n).collect::<String>()
    }}
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
