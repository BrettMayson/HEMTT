#![allow(clippy::unwrap_used)]

//! A realistic CBA style project, from `common::cba_project`, through every
//! stage: check, build, dev and release. The tests look at what is actually
//! produced, not just whether the command succeeded.

mod common;

use std::{
    io::{Cursor, Read},
    path::Path,
};

use common::{cba_project, files_with_extension, read, run, write};
use hemtt_config::rapify::Derapify;
use hemtt_pbo::ReadablePbo;
use hemtt_signing::{BIPublicKey, BISign};
use sealed_test::prelude::*;

fn open(path: impl AsRef<Path>) -> ReadablePbo<fs_err::File> {
    let path = path.as_ref();
    ReadablePbo::from(fs_err::File::open(path).unwrap())
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn names(pbo: &ReadablePbo<fs_err::File>) -> Vec<String> {
    pbo.files()
        .iter()
        .map(|h| h.filename().to_string())
        .collect()
}

fn file(pbo: &mut ReadablePbo<fs_err::File>, name: &str) -> Vec<u8> {
    let mut data = Vec::new();
    pbo.file(name)
        .unwrap()
        .unwrap_or_else(|| panic!("{name} is in the pbo"))
        .read_to_end(&mut data)
        .unwrap();
    data
}

/// The derapified `config.bin` of a PBO
fn config(pbo: &mut ReadablePbo<fs_err::File>) -> String {
    let bin = file(pbo, "config.bin");
    hemtt_config::Config::derapify(&mut Cursor::new(bin))
        .unwrap()
        .to_string()
}

#[sealed_test]
fn checks_clean() {
    let _directory = cba_project();
    run(&["check", "--error-on-all"])
        .succeeds()
        .does_not_print("warning[")
        .does_not_print("help[");
}

#[sealed_test]
fn build_layout() {
    let _directory = cba_project();
    run(&["build"]).succeeds();
    let pbos = files_with_extension(".hemttout/build", "pbo");
    assert_eq!(
        pbos,
        [
            Path::new(".hemttout/build/addons/tst_common.pbo"),
            Path::new(".hemttout/build/addons/tst_main.pbo"),
            Path::new(".hemttout/build/optionals/@tst_compat/addons/tst_compat.pbo"),
        ]
    );
    assert_eq!(read(".hemttout/build/mod.cpp"), read("mod.cpp"));
    assert_eq!(read(".hemttout/build/README.md"), read("README.md"));
}

#[sealed_test]
fn build_pbo_contents() {
    let _directory = cba_project();
    run(&["build"]).succeeds();
    let mut main = open(".hemttout/build/addons/tst_main.pbo");
    let files = names(&main);
    assert!(files.contains(&"config.bin".to_string()), "{files:?}");
    assert!(
        !files.contains(&"config.cpp".to_string()),
        "rapified: {files:?}"
    );
    assert!(files.contains(&"stringtable.bin".to_string()), "{files:?}");
    assert!(
        files.contains(&"functions\\fnc_hello.sqf".to_string()),
        "{files:?}"
    );
    assert!(
        files.contains(&"functions\\fnc_hello.sqfc".to_string()),
        "{files:?}"
    );
    assert!(
        !files.iter().any(|f| std::path::Path::new(f)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("psd"))),
        "excluded: {files:?}"
    );
    assert!(!files.iter().any(|f| f.contains("PBOPREFIX")), "{files:?}");
    assert_eq!(file(&mut main, "data\\notes.txt"), b"packed as is\n");
    assert_eq!(
        file(&mut main, "functions\\fnc_hello.sqf"),
        fs_err::read("addons/main/functions/fnc_hello.sqf").unwrap(),
        "sqf is packed as written, with its macros"
    );
    assert!(main.is_sorted().is_ok());
    assert_eq!(&main.gen_checksum().unwrap(), main.checksum());
}

#[sealed_test]
fn build_pbo_properties() {
    let _directory = cba_project();
    run(&["build"]).succeeds();
    for (path, prefix) in [
        (
            ".hemttout/build/addons/tst_main.pbo",
            "z\\tst\\addons\\main",
        ),
        (
            ".hemttout/build/addons/tst_common.pbo",
            "z\\tst\\addons\\common",
        ),
        (
            ".hemttout/build/optionals/@tst_compat/addons/tst_compat.pbo",
            "z\\tst\\addons\\compat",
        ),
    ] {
        let pbo = open(path);
        let properties = pbo.properties();
        assert_eq!(properties.get("prefix").unwrap(), prefix, "{path}");
        assert_eq!(properties.get("version").unwrap(), "1.2.3.4", "{path}");
        assert_eq!(properties.get("author").unwrap(), "Test Team", "{path}");
        assert!(properties.contains_key("hemtt"), "{path}");
        assert_eq!(
            properties.keys().next().unwrap(),
            "prefix",
            "prefix comes first: {path}"
        );
    }
}

#[sealed_test]
fn build_config_has_macros_expanded() {
    let _directory = cba_project();
    run(&["build"]).succeeds();
    let main = config(&mut open(".hemttout/build/addons/tst_main.pbo"));
    for expected in [
        "class tst_main {",
        "requiredVersion = 2.14;",
        "version = 1.2;",
        "versionAr[] = {1, 2, 3, 4};",
        r#"init = "call compile preprocessFileLineNumbers '\z\tst\addons\main\XEH_preInit.sqf'";"#,
        "class tst_main_display {",
        "w = 0.025;",
    ] {
        assert!(main.contains(expected), "`{expected}` is in\n{main}");
    }
    let common = config(&mut open(".hemttout/build/addons/tst_common.pbo"));
    assert!(common.contains("class tst_common"), "{common}");
    assert!(common.contains("\"tst_main\""), "{common}");
}

#[sealed_test]
fn build_stringtable_keys() {
    let _directory = cba_project();
    run(&["build"]).succeeds();
    let mut main = open(".hemttout/build/addons/tst_main.pbo");
    let bin = file(&mut main, "stringtable.bin");
    assert!(
        bin.windows(b"STR_tst_main_hello".len())
            .any(|w| w == b"STR_tst_main_hello"),
        "the key is in the binary stringtable"
    );
    assert!(bin.windows(b"Hallo %1".len()).any(|w| w == b"Hallo %1"));
}

#[sealed_test]
fn builds_are_reproducible() {
    let _directory = cba_project();
    run(&["build"]).succeeds();
    let first: Vec<_> = files_with_extension(".hemttout/build", "pbo")
        .iter()
        .map(|p| fs_err::read(p).unwrap())
        .collect();
    run(&["build"]).succeeds();
    let second: Vec<_> = files_with_extension(".hemttout/build", "pbo")
        .iter()
        .map(|p| fs_err::read(p).unwrap())
        .collect();
    assert!(
        first == second,
        "two builds of the same source are byte for byte equal"
    );
}

#[sealed_test]
fn removed_files_leave_the_next_build() {
    let _directory = cba_project();
    run(&["build"]).succeeds();
    assert!(
        names(&open(".hemttout/build/addons/tst_main.pbo"))
            .contains(&"data\\notes.txt".to_string())
    );
    fs_err::remove_file("addons/main/data/notes.txt").unwrap();
    run(&["build"]).succeeds();
    assert!(
        !names(&open(".hemttout/build/addons/tst_main.pbo"))
            .contains(&"data\\notes.txt".to_string())
    );
}

#[sealed_test]
fn removed_addons_leave_the_next_build() {
    let _directory = cba_project();
    run(&["build"]).succeeds();
    fs_err::remove_dir_all("addons/common").unwrap();
    run(&["build"]).succeeds();
    assert!(
        !Path::new(".hemttout/build/addons/tst_common.pbo").exists(),
        "a stale pbo would still be loaded by the game"
    );
}

#[sealed_test]
fn build_just_one_addon() {
    let _directory = cba_project();
    run(&["build", "--just", "main"]).succeeds();
    assert!(Path::new(".hemttout/build/addons/tst_main.pbo").exists());
    assert!(!Path::new(".hemttout/build/addons/tst_common.pbo").exists());
}

#[sealed_test]
fn build_just_unknown_addon_fails() {
    // A typo used to build nothing and succeed
    let _directory = cba_project();
    let result = run(&["build", "--just", "main", "--just", "nope"]);
    assert!(matches!(
        result.fails(),
        hemtt::Error::Addon(hemtt_workspace::addons::Error::AddonNotFound(name)) if name == "nope"
    ));
}

#[sealed_test]
fn dev_just_one_addon() {
    let _directory = cba_project();
    run(&["dev", "--just", "MAIN"]).succeeds();
    assert!(Path::new(".hemttout/dev/addons/tst_main.pbo").exists());
    assert!(!Path::new(".hemttout/dev/addons/tst_common.pbo").exists());
    run(&["dev", "--just", "nope"]).fails();
}

#[sealed_test]
fn version_comes_from_script_version() {
    let _directory = cba_project();
    run(&["value", "project.version"])
        .succeeds()
        .prints("1.2.3.4");
    write(
        "addons/main/script_version.hpp",
        "#define MAJOR 2\n#define MINOR 0\n#define PATCHLVL 1\n#define BUILD 0\n",
    );
    run(&["value", "project.version"])
        .succeeds()
        .prints("2.0.1.0");
    run(&["build"]).succeeds();
    assert_eq!(
        open(".hemttout/build/addons/tst_main.pbo")
            .properties()
            .get("version")
            .unwrap(),
        "2.0.1.0"
    );
}

// dev

#[sealed_test]
fn dev_links_into_the_sandboxed_arma_folder() {
    let directory = cba_project();
    run(&["dev"]).succeeds();
    let link = directory.join("arma3").join("z").join("tst");
    assert!(
        link.is_symlink(),
        "file patching link is made in the sandbox"
    );
    assert_eq!(
        fs_err::canonicalize(&link).unwrap(),
        fs_err::canonicalize(".hemttout/dev").unwrap()
    );
    assert!(Path::new(".hemttout/dev/addons/tst_main.pbo").exists());
    assert!(
        !Path::new(".hemttout/dev/optionals").exists(),
        "optionals are only built when asked for"
    );
}

#[sealed_test]
fn dev_with_optionals() {
    let _directory = cba_project();
    run(&["dev", "-O"]).succeeds();
    assert_eq!(
        files_with_extension(".hemttout/dev", "pbo").len(),
        3,
        "{:?}",
        files_with_extension(".hemttout/dev", "pbo")
    );
}

// release

#[sealed_test]
fn release_signs_everything() {
    let _directory = cba_project();
    run(&["release"]).succeeds();
    let main_key = BIPublicKey::read(
        &mut fs_err::File::open(".hemttout/release/keys/tst_1.2.3.4.bikey").unwrap(),
    )
    .unwrap();
    for name in ["tst_main", "tst_common"] {
        let path = format!(".hemttout/release/addons/{name}.pbo");
        let signature =
            BISign::read(&mut fs_err::File::open(format!("{path}.tst_1.2.3.4.bisign")).unwrap())
                .unwrap();
        main_key.verify(&mut open(&path), &signature).unwrap();
    }

    // Optionals in their own mod folder have their own key
    let optional = ".hemttout/release/optionals/@tst_compat";
    let key = BIPublicKey::read(
        &mut fs_err::File::open(format!("{optional}/keys/tst_1.2.3.4_tst_compat.bikey")).unwrap(),
    )
    .unwrap();
    let path = format!("{optional}/addons/tst_compat.pbo");
    let signature = BISign::read(
        &mut fs_err::File::open(format!("{path}.tst_1.2.3.4_tst_compat.bisign")).unwrap(),
    )
    .unwrap();
    key.verify(&mut open(&path), &signature).unwrap();
    assert!(
        main_key.verify(&mut open(&path), &signature).is_err(),
        "the main key can't verify the optional"
    );
}

#[sealed_test]
fn release_archive() {
    let _directory = cba_project();
    run(&["release"]).succeeds();
    let latest = fs_err::read("releases/tst-latest.zip").unwrap();
    let versioned = fs_err::read("releases/tst-1.2.3.4.zip").unwrap();
    assert_eq!(latest, versioned);

    let mut zip = zip::ZipArchive::new(Cursor::new(versioned)).unwrap();
    let mut names: Vec<String> = zip
        .file_names()
        .map(|n| n.unwrap().replace('\\', "/"))
        .filter(|n| !n.ends_with('/'))
        .collect();
    names.sort();
    assert_eq!(
        names,
        [
            "@tst/README.md",
            "@tst/addons/tst_common.pbo",
            "@tst/addons/tst_common.pbo.tst_1.2.3.4.bisign",
            "@tst/addons/tst_main.pbo",
            "@tst/addons/tst_main.pbo.tst_1.2.3.4.bisign",
            "@tst/keys/tst_1.2.3.4.bikey",
            "@tst/mod.cpp",
            "@tst/optionals/@tst_compat/addons/tst_compat.pbo",
            "@tst/optionals/@tst_compat/addons/tst_compat.pbo.tst_1.2.3.4_tst_compat.bisign",
            "@tst/optionals/@tst_compat/keys/tst_1.2.3.4_tst_compat.bikey",
        ]
    );

    // The pbos in the archive are the ones that were signed
    let mut packed = Vec::new();
    zip.by_name("@tst/addons/tst_main.pbo")
        .unwrap()
        .read_to_end(&mut packed)
        .unwrap();
    assert_eq!(
        packed,
        fs_err::read(".hemttout/release/addons/tst_main.pbo").unwrap()
    );
}

#[sealed_test]
fn release_fails_on_errors() {
    let _directory = cba_project();
    write("addons/main/broken.sqf", "private _a = ;\n");
    run(&["release"]).fails();
    assert!(!Path::new("releases/tst-1.2.3.4.zip").exists());
}

// diagnostics on the real project

#[sealed_test]
fn undefined_function_and_string() {
    let _directory = cba_project();
    write(
        "addons/common/bad.sqf",
        "#include \"script_component.hpp\"\n[] call EFUNC(main,missing);\nsystemChat localize LSTRING(missing);\n",
    );
    run(&["check"])
        .succeeds()
        .prints("Undefined Function: tst_main_fnc_missing")
        .does_not_print("tst_main_fnc_hello")
        .prints("L-L02");
}

#[sealed_test]
fn missing_file_from_macro() {
    let _directory = cba_project();
    write(
        "addons/common/bad.sqf",
        "#include \"script_component.hpp\"\nprivate _x = loadFile QPATHTOF(data\\nope.txt);\nsystemChat _x;\n",
    );
    run(&["check"])
        .succeeds()
        .prints("\\z\\tst\\addons\\common\\data\\nope.txt");
}

#[sealed_test]
fn missing_include_is_an_error_at_the_include() {
    let _directory = cba_project();
    write(
        "addons/common/bad.sqf",
        "#include \"script_component.hpp\"\n#include \"\\z\\tst\\addons\\main\\nope.hpp\"\n",
    );
    let result = run(&["check"]);
    result.fails();
    result
        .prints("error[PE12]")
        .prints("addons/common/bad.sqf:2");
}

#[sealed_test]
fn wrong_case_include_is_a_warning_everywhere() {
    // Arma ignores case, so this works in game, the build must succeed on every platform
    let _directory = cba_project();
    write(
        "addons/common/script_component.hpp",
        "#define COMPONENT common\n#include \"\\z\\tst\\addons\\main\\Script_Mod.hpp\"\n#include \"\\z\\tst\\addons\\MAIN\\script_macros.hpp\"\n",
    );
    run(&["build"])
        .succeeds()
        .prints("warning[PW4]")
        .prints("/addons/main/script_mod.hpp");
    assert!(Path::new(".hemttout/build/addons/tst_common.pbo").exists());
}

#[sealed_test]
fn wrong_case_relative_include_is_a_warning() {
    let _directory = cba_project();
    write(
        "addons/main/functions/fnc_hello.sqf",
        "#include \"..\\Script_Component.hpp\"\nparams [\"_name\"];\nsystemChat format [localize LSTRING(hello), _name];\n",
    );
    run(&["check"]).succeeds().prints("warning[PW4]");
}

#[sealed_test]
fn wrong_case_file_reference_is_not_missing() {
    let _directory = cba_project();
    write(
        "addons/common/bad.sqf",
        "#include \"script_component.hpp\"\nprivate _x = loadFile \"\\z\\tst\\addons\\main\\DATA\\Notes.txt\";\nsystemChat _x;\n",
    );
    run(&["check"])
        .succeeds()
        .does_not_print("nope")
        .does_not_print("L-S32");
}

#[sealed_test]
fn duplicate_prefix_fails() {
    let _directory = cba_project();
    write("addons/copy/$PBOPREFIX$", "z\\tst\\addons\\main");
    write("addons/copy/config.cpp", "class CfgPatches {};\n");
    run(&["check"]).fails();
}

#[sealed_test]
fn unused_stringtable_key_is_fine() {
    let _directory = cba_project();
    let table = read("addons/main/stringtable.xml").replace(
        "    </Package>",
        "        <Key ID=\"STR_tst_main_unused\">\n            <Original>Unused</Original>\n        </Key>\n    </Package>",
    );
    write("addons/main/stringtable.xml", table);
    run(&["check"]).succeeds().does_not_print("error[");
}

// prefix files

#[sealed_test]
fn prefix_file_variants() {
    let _directory = cba_project();
    for content in [
        "z\\tst\\addons\\main\n",
        "z\\tst\\addons\\main\r\n",
        "  z\\tst\\addons\\main  \n",
        // Mikero's format
        "prefix=z\\tst\\addons\\main\n",
    ] {
        write("addons/main/$PBOPREFIX$", content);
        run(&["build", "--just", "main"]).succeeds();
        assert_eq!(
            open(".hemttout/build/addons/tst_main.pbo")
                .properties()
                .get("prefix")
                .unwrap(),
            "z\\tst\\addons\\main",
            "{content:?}"
        );
    }
}

#[sealed_test]
fn prefix_file_names_ignore_case() {
    // `$PBOPREFIX$.txt` used to be missed on case-sensitive filesystems
    let _directory = cba_project();
    fs_err::remove_file("addons/main/$PBOPREFIX$").unwrap();
    for name in [
        "$PBOPREFIX$.txt",
        "$pboprefix$",
        "PboPrefix.txt",
        "$PBOPREFIX$.TXT",
    ] {
        write(format!("addons/main/{name}"), "z\\tst\\addons\\main");
        run(&["build", "--just", "main"]).succeeds();
        let pbo = open(".hemttout/build/addons/tst_main.pbo");
        assert_eq!(
            pbo.properties().get("prefix").unwrap(),
            "z\\tst\\addons\\main",
            "{name}"
        );
        assert!(
            !names(&pbo).iter().any(|f| f.eq_ignore_ascii_case(name)),
            "the prefix file is not packed: {name}"
        );
        fs_err::remove_file(format!("addons/main/{name}")).unwrap();
    }
}

#[sealed_test]
fn missing_prefix_file_fails() {
    let _directory = cba_project();
    fs_err::remove_file("addons/main/$PBOPREFIX$").unwrap();
    let result = run(&["build"]);
    assert!(matches!(
        result.fails(),
        hemtt::Error::Workspace(hemtt_workspace::Error::Addon(
            hemtt_workspace::addons::Error::PrefixMissing(name)
        )) if name == "main"
    ));
}

#[sealed_test]
fn leading_backslash_prefix_names_the_file() {
    let _directory = cba_project();
    write("addons/main/$PBOPREFIX$", "\\z\\tst\\addons\\main");
    let result = run(&["build"]);
    let error = result.fails();
    assert!(
        matches!(
            error,
            hemtt::Error::Workspace(hemtt_workspace::Error::Prefix(
                hemtt_common::prefix::Error::File { path, source }
            )) if path == "addons/main/$PBOPREFIX$"
                && matches!(**source, hemtt_common::prefix::Error::LeadingBackslash(_))
        ),
        "{error:?}"
    );
    let message = error.to_string();
    assert!(message.contains("addons/main/$PBOPREFIX$"), "{message}");
    assert!(message.contains("use `z\\tst\\addons\\main`"), "{message}");
}

// addon.toml

#[sealed_test]
fn addon_toml_excludes_are_relative_to_the_addon() {
    let _directory = cba_project();
    write(
        "addons/main/addon.toml",
        "[rapify]\nexclude = [\"extra/*.cpp\"]\n[files]\nexclude = [\"data/notes.txt\"]\n[properties]\niso = \"14001\"\n",
    );
    write("addons/main/extra/raw.cpp", "class Raw { x = 1; };\n");
    run(&["build"]).succeeds();
    let pbo = open(".hemttout/build/addons/tst_main.pbo");
    let files = names(&pbo);
    assert!(
        files.contains(&"extra\\raw.cpp".to_string()),
        "packed as is: {files:?}"
    );
    assert!(
        !files.contains(&"extra\\raw.bin".to_string()),
        "not rapified: {files:?}"
    );
    assert!(!files.contains(&"data\\notes.txt".to_string()), "{files:?}");
    assert!(!files.contains(&"addon.toml".to_string()), "{files:?}");
    assert_eq!(pbo.properties().get("iso").unwrap(), "14001");
    assert!(
        !open(".hemttout/build/addons/tst_common.pbo")
            .properties()
            .contains_key("iso"),
        "addon properties only apply to their addon"
    );
}

#[sealed_test]
fn addon_toml_rapify_disabled() {
    let _directory = cba_project();
    write("addons/common/addon.toml", "[rapify]\nenabled = false\n");
    // Not even preprocessed, so this would fail if it was
    write(
        "addons/common/config.cpp",
        "class CfgPatches { class tst_common { units[] = {}; requiredAddons[] = {}; }; };\n",
    );
    run(&["build"]).succeeds();
    let files = names(&open(".hemttout/build/addons/tst_common.pbo"));
    assert!(files.contains(&"config.cpp".to_string()), "{files:?}");
    assert!(!files.contains(&"config.bin".to_string()), "{files:?}");
}

// hooks

#[sealed_test]
fn pre_build_hook_sets_the_version() {
    let _directory = cba_project();
    write(
        ".hemtt/hooks/pre_build/01_version.rhai",
        "let f = HEMTT_VFS.join(\"addons\").join(\"main\").join(\"script_version.hpp\").create_file();\nf.write(\"#define MAJOR 9\\n#define MINOR 8\\n#define PATCHLVL 7\\n#define BUILD 6\\n\");\n",
    );
    run(&["release", "--no-sign"]).succeeds();
    assert_eq!(
        open(".hemttout/release/addons/tst_main.pbo")
            .properties()
            .get("version")
            .unwrap(),
        "9.8.7.6",
        "the version the hook wrote is used"
    );
    assert!(Path::new("releases/tst-9.8.7.6.zip").exists());
    assert!(
        read("addons/main/script_version.hpp").contains("MAJOR 1"),
        "the hook only changed the virtual file system"
    );
}

#[sealed_test]
fn failing_hook_fails_the_build() {
    let _directory = cba_project();
    write(".hemtt/hooks/pre_build/01_fail.rhai", "fatal(\"stop\");\n");
    run(&["build"]).fails();
    assert!(!Path::new(".hemttout/build/addons/tst_main.pbo").exists());
}

// project files

#[sealed_test]
fn glob_includes_are_released() {
    let _directory = cba_project();
    write("docs/guide.md", "g");
    write("docs/img/pic.png", "p");
    write(
        ".hemtt/project.toml",
        read(".hemtt/project.toml").replace(
            "include = [\"README.md\"]",
            "include = [\"README.md\", \"docs/**\"]",
        ),
    );
    run(&["release", "--no-sign"]).succeeds();
    assert_eq!(read(".hemttout/release/docs/img/pic.png"), "p");
    let zip = zip::ZipArchive::new(fs_err::File::open("releases/tst-latest.zip").unwrap()).unwrap();
    let names: Vec<_> = zip
        .file_names()
        .map(|n| n.unwrap().replace('\\', "/"))
        .collect();
    assert!(
        names.contains(&"@tst/docs/guide.md".to_string()),
        "{names:?}"
    );
    assert!(
        names.contains(&"@tst/docs/img/pic.png".to_string()),
        "{names:?}"
    );
}

// file contents

#[sealed_test]
fn unicode_survives_the_build() {
    let _directory = cba_project();
    write("addons/main/unicode.sqf", "systemChat \"Grüße 🎉 日本\";\n");
    write(
        "addons/main/config.cpp",
        read("addons/main/config.cpp").replace("Test Mod - Main", "Test Mod – Ünïcödé 🎉"),
    );
    run(&["build"]).succeeds();
    let mut pbo = open(".hemttout/build/addons/tst_main.pbo");
    assert_eq!(
        String::from_utf8(file(&mut pbo, "unicode.sqf")).unwrap(),
        "systemChat \"Grüße 🎉 日本\";\n"
    );
    assert!(config(&mut pbo).contains("Test Mod – Ünïcödé 🎉"));
}

#[sealed_test]
fn crlf_line_endings() {
    let _directory = cba_project();
    for f in [
        "addons/main/config.cpp",
        "addons/main/XEH_preInit.sqf",
        "addons/main/script_macros.hpp",
        "addons/main/script_component.hpp",
    ] {
        write(f, read(f).replace('\n', "\r\n"));
    }
    write(
        "addons/main/bad.sqf",
        "#include \"script_component.hpp\"\r\n\r\nprivate _a = 1;\r\nsystemChat str _undefined;\r\n",
    );
    run(&["build"])
        .succeeds()
        .prints("addons/main/bad.sqf:4:16");
    assert!(
        config(&mut open(".hemttout/build/addons/tst_main.pbo")).contains("class tst_main_display")
    );
}

#[sealed_test]
fn uppercase_extensions_are_compiled() {
    let _directory = cba_project();
    write("addons/main/UPPER.SQF", "systemChat str _undefinedUpper;\n");
    run(&["build"]).succeeds().prints("_undefinedUpper");
    let files = names(&open(".hemttout/build/addons/tst_main.pbo"));
    assert!(files.contains(&"UPPER.SQF".to_string()), "{files:?}");
    assert!(files.contains(&"UPPER.sqfc".to_string()), "{files:?}");
}

#[sealed_test]
fn nested_includes_and_spaces_in_paths() {
    let _directory = cba_project();
    write(
        "addons/main/my folder/my file.sqf",
        "#include \"..\\script_component.hpp\"\nsystemChat QGVAR(spaces);\n",
    );
    write(
        "addons/main/cfg/inner.hpp",
        "#include \"deeper\\value.hpp\"\n",
    );
    write(
        "addons/main/cfg/deeper/value.hpp",
        "class Inner { v = QGVAR(deep); };\n",
    );
    write(
        "addons/main/config.cpp",
        format!(
            "{}\n#include \"cfg\\inner.hpp\"\n",
            read("addons/main/config.cpp")
        ),
    );
    run(&["build"]).succeeds();
    let mut pbo = open(".hemttout/build/addons/tst_main.pbo");
    assert!(names(&pbo).contains(&"my folder\\my file.sqfc".to_string()));
    assert!(config(&mut pbo).contains("v = \"tst_main_deep\";"));
}

#[sealed_test]
fn mission_in_an_addon() {
    let _directory = cba_project();
    write(
        "addons/main/missions/test.VR/mission.sqm",
        "version=54;\nclass Mission { class Intel { briefingName=\"x\"; }; };\n",
    );
    write(
        "addons/main/missions/test.VR/init.sqf",
        "systemChat 'mission';\n",
    );
    run(&["build"]).succeeds();
    let mut pbo = open(".hemttout/build/addons/tst_main.pbo");
    let files = names(&pbo);
    assert!(
        files.contains(&"missions\\test.VR\\init.sqfc".to_string()),
        "{files:?}"
    );
    let sqm = file(&mut pbo, "missions\\test.VR\\mission.sqm");
    assert!(sqm.starts_with(b"\0raP"), "mission.sqm is rapified");
}

// project structure errors

#[sealed_test]
fn invalid_stringtable_fails() {
    let _directory = cba_project();
    write(
        "addons/main/stringtable.xml",
        "<Project name=\"TST\"><Package name=\"Main\"><Key ID=\"STR_tst_main_hello\"><Original>Hi</Original>\n",
    );
    let result = run(&["build"]);
    result.fails();
    result
        .prints("INVALID-STRINGTABLE")
        .prints("/addons/main/stringtable.xml");
}

// Needs a case-sensitive filesystem: on macOS and Windows both folders are the same folder
#[cfg(target_os = "linux")]
#[sealed_test]
fn addon_names_differing_by_case_fail() {
    let _directory = cba_project();
    for (folder, prefix) in [("Extra", "extra_a"), ("extra", "extra_b")] {
        write(
            format!("addons/{folder}/$PBOPREFIX$"),
            format!("z\\tst\\addons\\{prefix}"),
        );
        write(
            format!("addons/{folder}/config.cpp"),
            "class CfgPatches {};\n",
        );
    }
    let result = run(&["build"]);
    assert!(matches!(
        result.fails(),
        hemtt::Error::Workspace(hemtt_workspace::Error::Addon(
            hemtt_workspace::addons::Error::NameDuplicate(..)
        ))
    ));
}

#[sealed_test]
fn addon_in_addons_and_optionals_fails() {
    let _directory = cba_project();
    write("optionals/main/$PBOPREFIX$", "z\\tst\\addons\\mainopt");
    write("optionals/main/config.cpp", "class CfgPatches {};\n");
    let result = run(&["build"]);
    assert!(matches!(
        result.fails(),
        hemtt::Error::Workspace(hemtt_workspace::Error::Addon(
            hemtt_workspace::addons::Error::Duplicate(name)
        )) if name == "main"
    ));
}

#[sealed_test]
fn addon_without_config_is_packed() {
    let _directory = cba_project();
    write("addons/data/$PBOPREFIX$", "z\\tst\\addons\\data");
    write("addons/data/thing.txt", "x");
    run(&["build"]).succeeds();
    assert_eq!(
        names(&open(".hemttout/build/addons/tst_data.pbo")),
        ["thing.txt"]
    );
}
