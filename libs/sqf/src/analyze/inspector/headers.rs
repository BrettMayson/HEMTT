use arma3_wiki::model::{Function, Param, Value};
use hemtt_workspace::reporting::Processed;
use regex::{Match, Regex};
use std::sync::{Arc, OnceLock};

const MAX_ARG_INDEX: usize = 100;

#[must_use]
pub(crate) fn extract_from_header(processed: &Processed) -> Option<Arc<Function>> {
    for (path, source) in &processed.sources() {
        let filename = path.filename().to_lowercase();
        if (filename.starts_with("fnc_") || filename.starts_with("fn_"))
            && let Some(funcname) = filename.strip_suffix(".sqf")
            && let Ok(header) = try_match_styles(source, funcname)
        {
            return Some(header);
        }
    }
    None
}

#[derive(Debug)]
enum HeaderError {
    NoMatch,
    ArgsNoMatch,
    ArgsBadIndex,
}

fn try_match_styles(source: &str, filename: &str) -> Result<Arc<Function>, HeaderError> {
    match match_style(source, filename, get_regex_ace()) {
        Ok(header) => return Ok(header),
        Err(HeaderError::NoMatch) => {}
        Err(e) => println!("DEBUG: error {e:?} matching ACE header in source: {filename}"),
    }
    match match_style(source, filename, get_regex_cba()) {
        Ok(header) => return Ok(header),
        Err(HeaderError::NoMatch) => {}
        Err(e) => println!("DEBUG: error {e:?} matching CBA header in source: {filename}"),
    }
    // match match_style(source, filename, get_regex_clib()) {
    //     Ok(header) => return Ok(header),
    //     Err(HeaderError::NoMatch) => {}
    //     Err(e) => println!("DEBUG: error {e:?} matching CLib header in source: {filename}"),
    // }
    println!("DEBUG: no valid header found in source: {filename}");
    Err(HeaderError::NoMatch)
}

#[must_use]
fn match_value(input_low: &str) -> Option<Value> {
    const IGNORE_TAGS: &[&str] = &["br/", "&amp;", "&lt;", "&gt;"];
    fn str_to_type(input: &str) -> Value {
        if input.starts_with("array of ") {
            return Value::ArrayUnknown;
        }
        if input.contains(',') {
            // ToDo: could split on comma, but this tends to be used for array sub elements
            return Value::Anything;
        }
        match input {
            "any" | "anything" | "unknown" => Value::Anything,
            "array" | "vector" => Value::ArrayUnknown,
            "bool" | "boolean" => Value::Boolean,
            "code" => Value::Code,
            "config" => Value::Config,
            "control" => Value::Control,
            "display" => Value::Display,
            "group" => Value::Group,
            "hashmap" => Value::HashMapUnknown,
            "location" => Value::Location,
            "namespace" => Value::Namespace,
            "number" | "scalar" => Value::Number,
            "object" | "logic" => Value::Object,
            "side" => Value::Side,
            "string" | "text" => Value::String,
            "structuredtext" => Value::StructuredText,
            "nil" | "nothing" => Value::Nothing,
            _ => {
                println!("DEBUG: unknown type '{input}', defaulting to Anything");
                Value::Anything
            }
        }
    }
    let re_arg_types = Regex::new(r"(?s)<(?<type>[^>]*)>").expect("valid regex");
    let captures = re_arg_types.captures_iter(input_low).collect::<Vec<_>>();
    let mut types = captures
        .iter()
        .flat_map(|c| {
            let type_str = c.name("type").expect("re").as_str();
            type_str.split(" or ").collect::<Vec<_>>()
        })
        .filter(|i| !IGNORE_TAGS.contains(i))
        .map(str_to_type)
        .collect::<Vec<_>>();
    if types.is_empty() {
        return None;
    }
    if types.len() == 1 {
        return types.pop();
    }
    Some(Value::OneOf(
        types.into_iter().map(|t| (t, None)).collect::<Vec<_>>(),
    ))
}

fn parse_args(input: Option<Match<'_>>, re_arg_line: &Regex) -> Result<Vec<Param>, HeaderError> {
    let mut out = Vec::new();
    let input = input.ok_or(HeaderError::ArgsNoMatch)?.as_str();
    for line in input.lines() {
        let Some(caps) = re_arg_line.captures(line) else {
            if !out.is_empty() && (line.trim().is_empty() || line == " *") {
                break;
            }
            continue;
        };
        if let Some(m) = caps.name("index") {
            // regex has index, verify and pad if needed
            let Ok(arg_index) = m.as_str().parse::<usize>() else {
                return Err(HeaderError::ArgsBadIndex);
            };
            if arg_index < out.len() || arg_index > MAX_ARG_INDEX {
                return Err(HeaderError::ArgsBadIndex);
            }
            while out.len() < arg_index {
                out.push(Param::new(
                    format!("{}", out.len()),
                    Some("padding for skipped args".to_string()),
                    Value::Anything,
                    true,
                    None,
                    None,
                ));
            }
        }

        let arg_info = caps.name("info").map_or("#Unknown", |m| m.as_str()).trim();
        let arg_info_low = arg_info.to_lowercase();
        let arg_type = match_value(arg_info_low.as_str());
        let arg_optional = arg_type.is_none()
            || arg_info_low.contains("(optional")
            || arg_info_low.contains("(unused")
            || arg_info_low.contains("(not used")
            || arg_info_low.contains("(default");
        let arg_type = arg_type.unwrap_or(Value::Anything);
        out.push(Param::new(
            format!("{}", out.len()),
            Some(arg_info.to_string()),
            arg_type,
            arg_optional,
            None,
            None,
        ));
    }
    Ok(out)
}
#[must_use]
fn parse_return(input: Option<Match<'_>>) -> Option<Value> {
    fn allow_array(v: Value) -> Value {
        match v {
            Value::ArrayUnknown => v,
            Value::OneOf(mut vec) => {
                if !vec.contains(&(Value::ArrayUnknown, None)) {
                    vec.push((Value::ArrayUnknown, None));
                }
                Value::OneOf(vec)
            }
            _ => Value::OneOf(vec![(v, None), (Value::ArrayUnknown, None)]),
        }
    }
    let input = input?.as_str().to_lowercase();
    let Some(value) = match_value(input.as_str()) else {
        if input.contains("none") || input.contains("nothing") {
            // "None" often used to indicate no useable return value (may not be explicitly nil)
            return Some(Value::Nothing);
        }
        return None;
    };
    // multiple return types are often for an array, so allow arrays to reduce false positives
    if let Value::OneOf(_) = &value {
        return Some(allow_array(value));
    }
    if input.contains("0: ") {
        return Some(allow_array(value));
    }
    Some(value)
}
#[must_use]
#[allow(dead_code)]
fn parse_example(input: Option<Match<'_>>) -> String {
    input.map_or_else(String::new, |m| m.as_str().to_string())
}
#[must_use]
#[allow(dead_code)]
fn parse_public(input: Option<Match<'_>>) -> bool {
    if let Some(input) = input {
        let s = input.as_str().to_lowercase();
        return s.contains("true") || s.contains("yes");
    }
    false
}
#[must_use]
fn parse_function_name(input: &str, filename_low: &str) -> Option<String> {
    static RE_FUNC_FINDER: OnceLock<Regex> = OnceLock::new();
    debug_assert_eq!(filename_low, filename_low.to_lowercase());
    let re =
        RE_FUNC_FINDER.get_or_init(|| Regex::new(r"([\w\d_-]+_fnc_[\w\d_-]+)").expect("regex ok"));
    for cap in re.captures_iter(input) {
        let func_low = cap[1].to_lowercase();
        if func_low.ends_with(filename_low) {
            return Some(func_low);
        }
    }
    None
}
fn match_style(
    source: &str,
    filename: &str,
    regex: (&Regex, &Regex),
) -> Result<Arc<Function>, HeaderError> {
    let (re_header, re_arg_line) = regex;
    let Some(capture) = re_header.captures(source) else {
        return Err(HeaderError::NoMatch);
    };
    let args = parse_args(capture.name("arg"), re_arg_line)?;
    let ret = parse_return(capture.name("ret"));
    let name = parse_function_name(source, filename);
    let example = parse_example(capture.name("ex"));
    // let public = parse_public(capture.name("pub"));
    Ok(Arc::new(Function::new(name, ret, args, example)))
}
#[must_use]
fn get_regex_cba() -> (&'static Regex, &'static Regex) {
    static RE_HEADER: OnceLock<Regex> = OnceLock::new();
    static RE_ARG_LINE: OnceLock<Regex> = OnceLock::new();
    (RE_HEADER.get_or_init(|| {
        Regex::new(r"(?s)\/\*.*?Parameter[s]?:(?<arg>.+?)(?:Return[s]?:[\s\*]*(?<ret>.+?))?[\s\*]*(?:Examples:[\s\*]*(?<ex>.+?))?[\s\*]*(?:Public:(?<pub>.+?))?\*\/").expect("regex ok")
    }), RE_ARG_LINE.get_or_init(|| {
        Regex::new(r"^    _(?<info>.*)").expect("regex ok") // 4 spaces before _
    }))
}
#[must_use]
fn get_regex_ace() -> (&'static Regex, &'static Regex) {
    static RE_HEADER: OnceLock<Regex> = OnceLock::new();
    static RE_ARG_LINE: OnceLock<Regex> = OnceLock::new();
    (RE_HEADER.get_or_init(|| {
        Regex::new(r"(?s)\/\*.*?Argument[s]?:(?<arg>.+?)(?:Return Value[s]?:[\s\*]*(?<ret>.+?))?[\s\*]*(?:Example:[\s\*]*(?<ex>.+?))?[\s\*]*(?:Public:(?<pub>.+?))?\*\/").expect("regex ok")
    }), RE_ARG_LINE.get_or_init(|| {
        Regex::new(r"\* (?<index>\d*):(?<info>.*)").expect("regex ok") // `* 0: description`
    }))
}
// #[must_use]
// fn get_regex_clib() -> (&'static Regex, &'static Regex) {
//     static RE_HEADER: OnceLock<Regex> = OnceLock::new();
//     static RE_ARG_LINE: OnceLock<Regex> = OnceLock::new();
//     (RE_HEADER.get_or_init(|| {
//         Regex::new(r"(?s)\/\*.*?Parameter\(s\):(?<arg>.+?)(?:Return[s]?:[\s\*]*(?<ret>.+?))?[\s\*]*(?:Example[s]?:[\s\*]*(?<ex>.+?))?\*\/").expect("regex ok")
//     }), RE_ARG_LINE.get_or_init(|| {
//         Regex::new(r"    (?<index>\d*):(?<info>.*)").expect("regex ok") // `    0: description`
//     }))
// }

mod tests {
    #[test]

    pub fn test_header() {
        let source = r#"
/*
 * Arguments:
 * 0: "test" <ARRAY>
 *
 * Return Value:
 * X <BOOLEAN>
 */
"#;
        let h = super::try_match_styles(source, "fnc_test_header".to_lowercase().as_str());
        println!("Header: {h:?}");
        assert!(h.is_ok_and(|h| {
            h.ret()
                .is_some_and(|r| r == &arma3_wiki::model::Value::Boolean)
                && h.params()
                    .first()
                    .is_some_and(|a| a.typ() == &arma3_wiki::model::Value::ArrayUnknown)
        }));
    }
}
