use arma3_wiki::Wiki;

fn main() {
    let wiki = Wiki::load(true);

    let mut flow = Vec::with_capacity(500);

    for command in wiki.commands().raw().values() {
        let name = command.name();
        if name.contains(' ') || name.contains('%') || name.contains('_') || name.contains('+') {
            continue;
        }
        if !name.is_ascii() {
            continue;
        }
        let dest = if command.groups().iter().any(|x| x == "Program Flow") {
            &mut flow
        } else {
            continue;
        };
        dest.push(command.name());
    }

    flow.retain(|x| !["spawn", "call"].contains(x));

    let content = std::fs::read_to_string("languages-src/sqf.json").unwrap();
    let content = content.replace("$flow$", &flow.join("|"));
    std::fs::write("languages/sqf.json", content).unwrap();
}
