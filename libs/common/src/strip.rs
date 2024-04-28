pub trait StripInsensitive {
    fn strip_prefix_insensitive(&self, prefix: &str) -> Option<&str>;
}

impl StripInsensitive for str {
    fn strip_prefix_insensitive(&self, prefix: &str) -> Option<&str> {
        if self.len() < prefix.len() {
            return None;
        }
        if self[..prefix.len()].eq_ignore_ascii_case(prefix) {
            Some(&self[prefix.len()..])
        } else {
            None
        }
    }
}

impl StripInsensitive for String {
    fn strip_prefix_insensitive(&self, prefix: &str) -> Option<&str> {
        self.as_str().strip_prefix_insensitive(prefix)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_prefix_insensitive() {
        assert_eq!("foobar".strip_prefix_insensitive("foo"), Some("bar"));
        assert_eq!("foobar".strip_prefix_insensitive("FOO"), Some("bar"));
        assert_eq!("foobar".strip_prefix_insensitive("bar"), None);
        assert_eq!("foobar".strip_prefix_insensitive("baz"), None);
        assert_eq!("FoObAr".strip_prefix_insensitive("foo"), Some("bAr"));
        assert_eq!("FoObAr".strip_prefix_insensitive("FOO"), Some("bAr"));
        assert_eq!("FoObAr".strip_prefix_insensitive("bar"), None);
        assert_eq!("FoObAr".strip_prefix_insensitive("baz"), None);
    }
}
