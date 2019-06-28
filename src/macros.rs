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

// Colored Output

#[macro_export]
macro_rules! yellow {
    ($s:expr, $m:expr) => {
        crate::niceprintln!(yellow, $s, $m);
    }
}

#[macro_export]
macro_rules! blue {
    ($s:expr, $m:expr) => {
        crate::niceprintln!(blue, $s, $m);
    }
}

#[macro_export]
macro_rules! green {
    ($s:expr, $m:expr) => {
        crate::niceprintln!(green, $s, $m);
    }
}

#[macro_export]
macro_rules! cyan {
    ($s:expr, $m:expr) => {
        crate::niceprintln!(cyan, $s, $m);
    }
}

#[macro_export]
macro_rules! magenta {
    ($s:expr, $m:expr) => {
        crate::niceprintln!(magenta, $s, $m);
    }
}

#[macro_export]
macro_rules! red {
    ($s:expr, $m:expr) => {
        crate::niceprintln!(red, $s, $m);
    }
}

#[macro_export]
macro_rules! black {
    ($s:expr, $m:expr) => {
        crate::niceprintln!(black, $s, $m);
    }
}

#[macro_export]
macro_rules! white {
    ($s:expr, $m:expr) => {
        crate::niceprintln!(white, $s, $m);
    }
}

#[macro_export]
macro_rules! nicefmt {
    ($c:ident, $s:expr, $m:expr) => {{
        let status = $s.$c().bold();
        let spacer = crate::repeat!(" ", 10 - $s.len());
        let message = $m;
        crate::iformat!("{spacer}{status} {message}", spacer, status, message)
    }}
}

#[macro_export]
macro_rules! niceprintln {
    ($c:ident, $s:expr, $m:expr) => {
        let r = crate::nicefmt!($c, $s, $m);
        println!("{}", r);
    }
}

#[macro_export]
macro_rules! warn {
    ($s:expr, $m:expr) => {
        use colored::*;
        let style = "warning".yellow().bold();
        let status = $s.bold();
        let message = $m;
        crate::iprintln!("{style}: {status}\n    {message}\n", style, status, message);
    }
}

#[macro_export]
macro_rules! error {
    ($s:expr) => {{
        use colored::*;
        let style = "error".red().bold();
        let status = $s;
        crate::iprintln!("{style}: {status}\n", style, status);
    }}
}

#[macro_export]
macro_rules! errormessage {
    ($s:expr, $m:expr) => {
        let status = $s;
        let message = $m;
        crate::error!(crate::iformat!("{status}\n    {message}", status, message));
    }
}

#[macro_export]
macro_rules! filewarn {
    ($e:expr) => {{
        use colored::*;
        let style = "warning".yellow().bold();
        let status = &$e.error.bold();
        let point = filepointer!($e);
        crate::iprintln!("{style}: {status}\n{point}", style, status, point)
    }}
}

#[macro_export]
macro_rules! fileerror {
    ($e:expr) => {
        use colored::*;
        let style = "error".red().bold();
        let status = &$e.error.bold();
        let point = filepointer!($e);
        crate::iprintln!("{style}: {status}\n{point}", style, status, point)
    }
}

#[macro_export]
macro_rules! filepointer {
    ($e:expr) => {{
        let content = &$e.content;
        let arrow = "-->".blue().bold();
        let sep = "|".blue().bold();
        //let end = "=".blue().bold();
        let file = &$e.file;
        let line = &$e.line.unwrap().to_string().blue().bold();
        let space = repeat!(" ", line.len() + 2);
        crate::iformat!("  {arrow} {file}\n{space}{sep}\n {line} {sep} {content}\n{space}{sep}\n", arrow, file, sep, line, space, content)
    }}
}


// Generic

#[macro_export]
macro_rules! ask {
    ($q:expr) => {{
        let mut x = String::new();
        while x.is_empty() {
            x = if let question::Answer::RESPONSE(n) = question::Question::new($q).ask().unwrap() { n } else { unreachable!() };
        }
        x
    }};
    ($q:expr, $d:expr) => {{
        let mut x = String::new();
        while x.is_empty() {
            x = if let question::Answer::RESPONSE(n) = question::Question::new($q)
                .default(question::Answer::RESPONSE($d.to_owned())).show_defaults().ask().unwrap() { n } else { unreachable!() };
        }
        x
    }};
}

#[macro_export]
macro_rules! repeat {
    ($s:expr, $n:expr) => {{
        &std::iter::repeat($s).take($n).collect::<String>()
    }}
}

#[macro_export]
macro_rules! fill_space {
    ($c:expr, $s:expr, $n:expr) => {{
        let s = ($s as i32) - ($n.len() as i32);
        let n = (if s <= 0 {0} else {s}) as usize;
        let t = if n == 0 { &$n[..$s] } else { $n };
        format!("{}{}", t, repeat!($c, n))
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
