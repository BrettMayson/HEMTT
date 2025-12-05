use arma3_wiki::Wiki;

pub fn run() {
    let wiki = Wiki::load(true);

    let mut flow = Vec::with_capacity(500);
    let mut commands = Vec::with_capacity(3000);

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

    let highlight = fs_err::read_to_string("book-gen/highlight.js").unwrap();

    let highlight = highlight.replace("$FLOW$", &format!("'{}'", flow.join("','")));
    let highlight = highlight.replace("$COMMANDS$", &format!("'{}'", commands.join("','")));

    fs_err::write("book/highlight.js", highlight).unwrap();
}
