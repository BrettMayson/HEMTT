use arma3_wiki::Wiki;

const SPECIAL_COMMANDS: [&str; 11] = [
    "call",
    "callExtension",
    "compile",
    "compileFinal",
    "exec",
    "execFSM",
    "execVM",
    "private",
    "spawn",
    "true",
    "false",
];

fn main() {
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

    commands.retain(|x| !SPECIAL_COMMANDS.contains(x));
    commands.sort_unstable();
    let commands = commands.into_iter().map(regex_escape).collect::<Vec<_>>();

    let content = std::fs::read_to_string("languages-src/sqf.json").unwrap();
    let mut content = content.replace("$flow$", &flow.join("|"));
    const CHUNKS: usize = 4;
    let chunked = commands.len() / CHUNKS;
    for i in 0..CHUNKS {
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

fn regex_escape(s: &str) -> String {
    s.replace(
        [
            '\\', '.', '*', '+', '?', '(', ')', '[', ']', '{', '}', '^', '$', '|',
        ],
        "\\\\$&",
    )
}
