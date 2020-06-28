#[macro_use]
mod fs;

// String Interpolation

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

// #[macro_export]
// macro_rules! filewarn {
//     ($e:expr) => {{
//         let status = &$e.error;
//         let point = filepointer!($e);
//         warn!("{}: {}", status, point)
//     }};
// }

// #[macro_export]
// macro_rules! fileerror {
//     ($e:expr) => {
//         let status = &$e.error;
//         let point = filepointer!($e);
//         error!("{}: {}", status, point)
//     };
// }

// #[macro_export]
// macro_rules! filepointer {
//     ($e:expr) => {{
//         let content = &$e.content;
//         let arrow = "-->".blue().bold();
//         let sep = "|".blue().bold();
//         //let end = "=".blue().bold();
//         let file = &$e.file;
//         let line = &$e.line.unwrap().to_string().blue().bold();
//         let space = repeat!(" ", line.len() + 2);
//         crate::iformat!(
//             "  {arrow} {file}\n{space}{sep}\n {line} {sep} {content}\n{space}{sep}\n",
//             arrow,
//             file,
//             sep,
//             line,
//             space,
//             content
//         )
//     }};
// }

// Generic

#[macro_export]
macro_rules! ask {
    ($q:expr) => {{
        let mut x = String::new();
        while x.is_empty() {
            x = if let question::Answer::RESPONSE(n) = question::Question::new($q).ask().unwrap() {
                n
            } else {
                unreachable!()
            };
        }
        x
    }};
    ($q:expr, $d:expr) => {{
        let mut x = String::new();
        while x.is_empty() {
            x = if let question::Answer::RESPONSE(n) = question::Question::new($q)
                .default(question::Answer::RESPONSE($d.to_owned()))
                .show_defaults()
                .ask()
                .unwrap()
            {
                n
            } else {
                unreachable!()
            };
        }
        x
    }};
}

#[macro_export]
macro_rules! repeat {
    ($s:expr, $n:expr) => {{
        &std::iter::repeat($s).take($n).collect::<String>()
    }};
}

#[macro_export]
macro_rules! fill_space {
    ($c:expr, $s:expr, $n:expr) => {{
        let s = ($s as i32) - ($n.len() as i32);
        let n = (if s <= 0 { 0 } else { s }) as usize;
        let t = if n == 0 { &$n[..$s] } else { $n };
        format!("{}{}", t, repeat!($c, n))
    }};
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
