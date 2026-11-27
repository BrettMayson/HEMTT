#![allow(dead_code, clippy::unwrap_used)]

//! Shared harness for the CLI integration tests.
//!
//! Each test copies a fixture workspace into a temporary directory and runs
//! the CLI in it with [`run`]. The returned [`Run`] holds both the result and
//! everything the command printed, so a test can check the exit status and
//! the output together.
//!
//! Debug builds of `hemtt::execute` return an error instead of exiting the
//! process when a command fails, which is what makes the exit status
//! observable here. Use `#[sealed_test]` on every test, since commands change
//! the current directory and set up global state.

use std::path::{Path, PathBuf};

use clap::Parser;
use hemtt::{Cli, Error};
use hemtt_common::capture::OutputCapture;
use hemtt_test::directory::TemporaryDirectory;

/// Copy a fixture from `bin/tests/` into a temporary directory and enter it
pub fn fixture(name: &str) -> TemporaryDirectory {
    TemporaryDirectory::copy(&fixture_path(name))
}

/// The path of a fixture in `bin/tests/`
pub fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(format!("{}/tests/{name}/", env!("CARGO_MANIFEST_DIR")))
}

/// The outcome of a CLI invocation
pub struct Run {
    pub args: Vec<String>,
    pub result: Result<(), Error>,
    pub output: String,
}

impl Run {
    /// Assert the command succeeded
    #[track_caller]
    pub fn succeeds(&self) -> &Self {
        if let Err(e) = &self.result {
            panic!(
                "`{}` failed: {e}\n--- output ---\n{}",
                self.args.join(" "),
                self.output
            );
        }
        self
    }

    /// Assert the command failed, returning the error
    #[track_caller]
    pub fn fails(&self) -> &Error {
        match &self.result {
            Ok(()) => panic!(
                "`{}` succeeded, but should have failed\n--- output ---\n{}",
                self.args.join(" "),
                self.output
            ),
            Err(e) => e,
        }
    }

    /// Assert the output contains `needle`
    #[track_caller]
    pub fn prints(&self, needle: &str) -> &Self {
        assert!(
            self.output.contains(needle),
            "`{}` did not print `{needle}`\n--- output ---\n{}",
            self.args.join(" "),
            self.output
        );
        self
    }

    /// Assert the output does not contain `needle`
    #[track_caller]
    pub fn does_not_print(&self, needle: &str) -> &Self {
        assert!(
            !self.output.contains(needle),
            "`{}` printed `{needle}`\n--- output ---\n{}",
            self.args.join(" "),
            self.output
        );
        self
    }
}

/// Run the CLI with `args`, without the leading `hemtt`
pub fn run(args: &[&str]) -> Run {
    let args: Vec<String> = std::iter::once("hemtt")
        .chain(args.iter().copied())
        .map(String::from)
        .collect();
    let cli = Cli::try_parse_from(&args)
        .unwrap_or_else(|e| panic!("`{}` did not parse: {e}", args.join(" ")));
    let capture = OutputCapture::new();
    let result = hemtt::execute(&cli);
    let output = capture.finish();
    Run {
        args,
        result,
        output,
    }
}

/// Write a file, creating its parent folders
pub fn write(path: impl AsRef<Path>, content: impl AsRef<[u8]>) {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs_err::create_dir_all(parent).unwrap();
    }
    fs_err::write(path, content).unwrap();
}

/// Read a file as a string
pub fn read(path: impl AsRef<Path>) -> String {
    fs_err::read_to_string(path).unwrap()
}

/// Every file under `root` with the given extension, sorted
pub fn files_with_extension(root: impl AsRef<Path>, ext: &str) -> Vec<PathBuf> {
    let mut files: Vec<_> = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .map(walkdir::DirEntry::into_path)
        .filter(|p| p.extension().is_some_and(|e| e == ext))
        .collect();
    files.sort();
    files
}

/// Write a realistic CBA style project into the current directory
///
/// - `addons/main` with the shared macros, version, `XEH_PREP` functions and a stringtable
/// - `addons/common` which depends on main and uses its macros and functions
/// - `optionals/compat` with its own `CfgPatches`
/// - `include/a3` with a header included by absolute path
/// - root files that are copied to the build (`mod.cpp`, `README.md`)
///
/// It checks with no diagnostics, so a test can add one problem and look for it.
pub fn cba_project() -> TemporaryDirectory {
    let directory = TemporaryDirectory::new();
    write(
        ".hemtt/project.toml",
        r#"name = "Test Mod"
prefix = "tst"
mainprefix = "z"

[version]
git_hash = 0

[files]
include = ["README.md"]
exclude = ["*.psd"]

[properties]
author = "Test Team"
"#,
    );
    write("mod.cpp", "name = \"Test Mod\";\n");
    write("README.md", "# Test Mod\n");
    write(
        "include/a3/ui_f/hpp/defineCommonGrids.inc",
        "#define GUI_GRID_W 0.025\n",
    );

    // main
    write("addons/main/$PBOPREFIX$", "z\\tst\\addons\\main");
    write(
        "addons/main/script_version.hpp",
        "#define MAJOR 1\n#define MINOR 2\n#define PATCHLVL 3\n#define BUILD 4\n",
    );
    write(
        "addons/main/script_mod.hpp",
        "#define MAINPREFIX z\n#define PREFIX tst\n#include \"script_version.hpp\"\n#define VERSION MAJOR.MINOR\n#define VERSION_AR MAJOR,MINOR,PATCHLVL,BUILD\n#define REQUIRED_VERSION 2.14\n",
    );
    write(
        "addons/main/script_macros.hpp",
        r"#define DOUBLES(var1,var2) var1##_##var2
#define TRIPLES(var1,var2,var3) var1##_##var2##_##var3
#define QUOTE(var1) #var1
#define ADDON DOUBLES(PREFIX,COMPONENT)
#define GVAR(var1) DOUBLES(ADDON,var1)
#define QGVAR(var1) QUOTE(GVAR(var1))
#define FUNC(var1) TRIPLES(ADDON,fnc,var1)
#define EFUNC(var1,var2) TRIPLES(DOUBLES(PREFIX,var1),fnc,var2)
#define PATHTOF(var1) \MAINPREFIX\PREFIX\addons\COMPONENT\var1
#define QPATHTOF(var1) QUOTE(PATHTOF(var1))
#define LSTRING(var1) QUOTE(TRIPLES(STR,ADDON,var1))
#define PREP(fncName) FUNC(fncName) = compile preprocessFileLineNumbers QPATHTOF(functions\DOUBLES(fnc,fncName).sqf)
",
    );
    write(
        "addons/main/script_component.hpp",
        "#define COMPONENT main\n#include \"script_mod.hpp\"\n#include \"script_macros.hpp\"\n",
    );
    write(
        "addons/main/config.cpp",
        r#"#include "script_component.hpp"
#include "\a3\ui_f\hpp\defineCommonGrids.inc"

class CfgPatches {
    class ADDON {
        name = "Test Mod - Main";
        units[] = {};
        weapons[] = {};
        requiredVersion = REQUIRED_VERSION;
        requiredAddons[] = {};
        author = "Test Team";
        version = VERSION;
        versionAr[] = {VERSION_AR};
    };
};

class Extended_PreInit_EventHandlers {
    class ADDON {
        init = QUOTE(call compile preprocessFileLineNumbers 'PATHTOF(XEH_preInit.sqf)');
    };
};

class RscTitles {
    class GVAR(display) {
        w = GUI_GRID_W;
    };
};
"#,
    );
    write("addons/main/XEH_PREP.hpp", "PREP(hello);\n");
    write(
        "addons/main/XEH_preInit.sqf",
        "#include \"script_component.hpp\"\n#include \"XEH_PREP.hpp\"\nGVAR(ready) = true;\n",
    );
    write(
        "addons/main/functions/fnc_hello.sqf",
        "#include \"..\\script_component.hpp\"\nparams [\"_name\"];\nsystemChat format [localize LSTRING(hello), _name];\n",
    );
    write(
        "addons/main/stringtable.xml",
        r#"<?xml version="1.0" encoding="utf-8"?>
<Project name="TST">
    <Package name="Main">
        <Key ID="STR_tst_main_hello">
            <Original>Hello %1</Original>
            <English>Hello %1</English>
            <German>Hallo %1</German>
        </Key>
    </Package>
</Project>
"#,
    );
    write("addons/main/data/notes.txt", "packed as is\n");
    write("addons/main/data/source.psd", "excluded");

    // common
    write("addons/common/$PBOPREFIX$", "z\\tst\\addons\\common");
    write(
        "addons/common/script_component.hpp",
        "#define COMPONENT common\n#include \"\\z\\tst\\addons\\main\\script_mod.hpp\"\n#include \"\\z\\tst\\addons\\main\\script_macros.hpp\"\n",
    );
    write(
        "addons/common/config.cpp",
        r#"#include "script_component.hpp"

class CfgPatches {
    class ADDON {
        name = "Test Mod - Common";
        units[] = {};
        weapons[] = {};
        requiredVersion = REQUIRED_VERSION;
        requiredAddons[] = {"tst_main"};
        author = "Test Team";
    };
};

class Extended_PostInit_EventHandlers {
    class ADDON {
        init = QUOTE(call compile preprocessFileLineNumbers 'PATHTOF(XEH_postInit.sqf)');
    };
};
"#,
    );
    write(
        "addons/common/XEH_postInit.sqf",
        "#include \"script_component.hpp\"\n[\"world\"] call EFUNC(main,hello);\nprivate _ready = missionNamespace getVariable [QGVAR(ready), false];\nif (_ready) then { systemChat str _ready; };\n",
    );

    // optional
    write("optionals/compat/$PBOPREFIX$", "z\\tst\\addons\\compat");
    write(
        "optionals/compat/config.cpp",
        r#"class CfgPatches {
    class tst_compat {
        name = "Test Mod - Compat";
        units[] = {};
        weapons[] = {};
        requiredVersion = 2.14;
        requiredAddons[] = {"tst_main"};
    };
};
"#,
    );
    directory
}
