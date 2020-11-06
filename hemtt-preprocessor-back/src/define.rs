use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq)]
pub struct Define {
    pub ident: String,
    pub call: bool,
    pub args: Option<Vec<String>>,
    pub statement: String,
}

impl TryFrom<String> for Define {
    type Error = ();
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let value = value.trim_matches(|c| c == '\n' || c == ' ');
        println!("Trying from: `{}`", value);
        let sections = value.split(' ').collect::<Vec<&str>>();
        if let Some(ident) = sections.get(0) {
            if let Some(index) = ident.find('(') {
                if let Some(endex) = ident.find(')') {
                    let mut vargs = Vec::new();
                    ident[index + 1..endex].split(',').for_each(|a| {
                        vargs.push(a.to_string());
                    });
                    Ok(Define {
                        ident: ident[..index].trim_matches(|c| c == '\n' || c == ' ').to_string(),
                        call: true,
                        args: Some(vargs),
                        statement: value[ident.len() + 1..].trim_matches(|c| c == '\n' || c == ' ').to_owned(),
                    })
                } else {
                    Err(())
                }
            } else {
                Ok(Define {
                    ident: (*ident).trim_matches(|c| c == '\n' || c == ' ').to_string(),
                    call: false,
                    args: None,
                    statement: value[ident.len() + 1..].trim_matches(|c| c == '\n' || c == ' ').to_owned(),
                })
            }
        } else {
            Err(())
        }
    }
}
