pub struct Vars<'a> {
    pub addon: &'a str,
}
impl<'a> Vars<'a> {
    pub fn vec(&self) -> Vec<(&'a str, &'a str)> {
        vec![("addon", self.addon)]
    }
}

pub fn replace<S: Into<String>>(vars: &Vars, content: S) -> String {
    let mut ret = content.into();
    for (k, v) in vars.vec().iter() {
        ret = ret.replace(&format!("%%{}%%", k), v);
        ret = ret.replace(&format!("%%{}%%", k.to_uppercase()), &v.to_uppercase());
    }
    ret
}

#[cfg(test)]
mod tests {
    #[test]
    fn basic_replace() {
        let vars = super::Vars { addon: "something" };
        assert_eq!(
            "path/to/something",
            super::replace(&vars, "path/to/%%addon%%"),
        )
    }

    #[test]
    fn basic_replace_cases() {
        let vars = super::Vars { addon: "text" };
        assert_eq!("path/to/text", super::replace(&vars, "path/to/%%addon%%"),);
        assert_eq!("path/to/TEXT", super::replace(&vars, "path/to/%%ADDON%%"),)
    }
}
