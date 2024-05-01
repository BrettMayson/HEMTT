use arma3_wiki::Wiki;

fn main() {
    let wiki = Wiki::load();

    let mut flow = Vec::with_capacity(500);
    let mut commands = Vec::with_capacity(3000);

    for command in wiki.commands().values() {
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
            &mut commands
        };
        dest.push(command.name());
    }

    // Remove special commands
    commands.retain(|x| {
        ![
            "call",
            "callExtension",
            "compile",
            "compileFinal",
            "exec",
            "execFSM",
            "execVM",
            "private",
            "spawn",
        ]
        .contains(x)
    });

    let content = std::fs::read_to_string("languages-src/sqf.json").unwrap();
    let mut content = content.replace("$flow$", &flow.join("|"));
    let chunked = commands.len() / 6;
    for i in 0..6 {
        let start = i * chunked;
        let end = if i == 5 {
            commands.len()
        } else {
            (i + 1) * chunked
        };
        content = content.replace(
            &format!("$commands{}$", i + 1),
            &commands[start..end].join("|"),
        );
    }
    std::fs::write("languages/sqf.json", content).unwrap();
}
