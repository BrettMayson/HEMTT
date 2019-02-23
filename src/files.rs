use reqwest;
use colored::*;

use std::fs;
use std::fs::File;
use std::io::{Read, Write, Error};
use std::path::Path;

use crate::project;

pub fn clear_pbos(p: &project::Project) -> Result<(), Error> {
    println!("  {} PBOs", "Cleaning".yellow().bold());
    for entry in fs::read_dir("addons")? {
        let entry = entry?;
        if !entry.path().is_dir() { continue }
        let name = entry.file_name().into_string().unwrap();
        clear_pbo(&p, &name)?;
    }
    Ok(())
}

pub fn clear_pbo(p: &project::Project, name: &String) -> Result<(), Error> {
    if Path::new(&format!("addons/{}_{}.pbo", p.prefix, name)).exists() {
        fs::remove_file(&format!("addons/{}_{}.pbo", p.prefix, name))?;
    }
    Ok(())
}

pub fn clear_release(version: &String) -> Result<(), Error> {
    if Path::new(&format!("releases/{}", version)).exists() {
        println!("  {} old release v{}", "Cleaning".yellow().bold(), version);
        fs::remove_dir_all(format!("releases/{}", version))?;
    }
    Ok(())
}

pub fn clear_releases() -> Result<(), Error> {
    println!("  {} all releases", "Cleaning".yellow().bold());
    if Path::new("releases").exists() {
        fs::remove_dir_all("releases")?;
    }
    Ok(())
}

pub fn modcpp(p: &project::Project) -> Result<(), std::io::Error> {
    let mut out = File::create("mod.cpp")?;
    out.write_fmt(
        format_args!("name = \"{}\";\ndir = \"@{}\";\nauthor = \"{}\";\n",
            p.name,
            p.prefix,
            p.author
        )
    )?;
    Ok(())
}

pub fn scriptmodhpp(p: &project::Project) -> Result<(), std::io::Error> {
    if !Path::new("addons/main").exists() {
        create_addon(&"main".to_owned(), &p)?;
    }
    let mut out = File::create("addons/main/script_mod.hpp")?;
    out.write_fmt(
        format_args!(
            "#define MAINPREFIX z\n#define PREFIX {}\n\n#include \"script_version.hpp\"\n\n#define VERSION MAJOR.MINOR.PATCH.BUILD\n#define VERSION_AR MAJOR,MINOR,PATCH,BUILD\n\n#define REQUIRED_VERSION 1.88\n",
            p.prefix
        )
    )?;
    Ok(())
}

pub fn scriptversionhpp(p: &project::Project) -> Result<(), std::io::Error> {
    if !Path::new("addons/main").exists() {
        create_addon(&"main".to_owned(), &p)?;
    }
    let mut out = File::create("addons/main/script_version.hpp")?;
    out.write_all(
        b"#define MAJOR 0\n#define MINOR 1\n#define PATCH 0\n#define BUILD 0\n"
    )?;
    Ok(())
}

pub fn scriptmacroshpp(p: &project::Project) -> Result<(), std::io::Error> {
    if !Path::new("addons/main").exists() {
        create_addon(&"main".to_owned(), &p)?;
    }
    let mut out = File::create("addons/main/script_macros.hpp")?;
    out.write_all(
br#"#include "\x\cba\addons\main\script_macros_common.hpp"
#define DFUNC(var1) TRIPLES(ADDON,fnc,var1)
#ifdef DISABLE_COMPILE_CACHE
    #undef PREP
    #define PREP(fncName) DFUNC(fncName) = compile preprocessFileLineNumbers QPATHTOF(functions\DOUBLES(fnc,fncName).sqf)
#else
    #undef PREP
    #define PREP(fncName) [QPATHTOF(functions\DOUBLES(fnc,fncName).sqf), QFUNC(fncName)] call CBA_fnc_compileFunction
#endif
"#
    )?;
    Ok(())
}

pub fn script_component(addon: &String, p: &project::Project) -> Result<(), std::io::Error> {
    if !Path::new(format!("addons/{}", &addon).as_str()).exists() {
        create_addon(&addon, &p)?;
    }
    let mut out = File::create(format!("addons/{}/script_component.hpp", addon))?;
    out.write_fmt(format_args!(
r#"#define COMPONENT {0}
#include "\z\{2}\addons\main\script_mod.hpp"

// #define DEBUG_MODE_FULL
// #define DISABLE_COMPILE_CACHE

#ifdef DEBUG_ENABLED_{1}
    #define DEBUG_MODE_FULL
#endif
    #ifdef DEBUG_SETTINGS_{1}
    #define DEBUG_SETTINGS DEBUG_SETTINGS_{1}
#endif

#include "\z\{2}\addons\main\script_macros.hpp"
"#,
        addon, addon.to_uppercase(), p.prefix
    ))?;
    Ok(())
}

pub fn pboprefix(addon: &String, p: &project::Project) -> Result<(), std::io::Error> {
    if !Path::new(format!("addons/{}", &addon).as_str()).exists() {
        create_addon(&addon, &p)?;
    }
    let mut out = File::create(format!("addons/{}/$PBOPREFIX$", addon))?;
    out.write_fmt(
        format_args!("z\\{}\\addons\\{}", p.prefix, addon)
    )?;
    Ok(())
}

pub fn configcpp(addon: &String, p: &project::Project) -> Result<(), std::io::Error> {
    if !Path::new(format!("addons/{}", &addon).as_str()).exists() {
        create_addon(&addon, &p)?;
    }
    let mut out = File::create(format!("addons/{}/config.cpp", addon))?;
    out.write_fmt(format_args!(
r#"#include "script_component.hpp"
    class CfgPatches {{
        class ADDON {{
            name = COMPONENT;
            units[] = {{}};
            weapons[] = {{}};
            requiredVersion = REQUIRED_VERSION;
            requiredAddons[] = {{}};
            author = "{}";
            VERSION_CONFIG;
        }};
    }};
"#,
        p.author)
    )?;

    if addon != "main" {
        out.write_all(b"\n\n#include \"CfgEventHandlers.hpp\"")?;
    }
    Ok(())
}

pub fn xeh(addon: &String, p: &project::Project) -> Result<(), std::io::Error> {
    if !Path::new(format!("addons/{}", &addon).as_str()).exists() {
        create_addon(&addon, &p)?;
    }
    fs::create_dir(format!("addons/{}/functions", addon))?;
    let mut out = File::create(format!("addons/{}/functions/script_component.hpp", addon))?;
    out.write_fmt(format_args!(r#"#include "\z\{}\addons\{}\script_component.hpp""#, p.prefix, addon))?;
    File::create(format!("addons/{}/XEH_PREP.hpp", addon))?;
    let mut out = File::create(format!("addons/{}/XEH_postInit.sqf", addon))?;
    out.write_all(br#"#include "script_component.hpp""#)?;
    let mut out = File::create(format!("addons/{}/XEH_preInit.sqf", addon))?;
    out.write_all(
br#"#include "script_component.hpp"
ADDON = false;
#include "XEH_PREP.hpp"
ADDON = true;
"#
    )?;
    let mut out = File::create(format!("addons/{}/XEH_preStart.sqf", addon))?;
    out.write_all(
br#"#include "script_component.hpp"
#include "XEH_PREP.hpp"
"#
    )?;
    let mut out = File::create(format!("addons/{}/CfgEventHandlers.hpp", addon))?;
    out.write_all(
br#"class Extended_PreStart_EventHandlers {
    class ADDON {
        init = QUOTE(call COMPILE_FILE(XEH_preStart));
    };
};
class Extended_PreInit_EventHandlers {
    class ADDON {
        init = QUOTE(call COMPILE_FILE(XEH_preInit));
    };
};
class Extended_PostInit_EventHandlers {
    class ADDON {
        init = QUOTE(call COMPILE_FILE(XEH_postInit));
    };
};
"#
    )?;
    Ok(())
}

pub fn create_include() -> Result<(), std::io::Error> {
    println!(" {} script_macros_common.hpp", "Downloading".green().bold());
    if !Path::new("include/x/cba/addons/main").exists() {
        fs::create_dir_all("include/x/cba/addons/main")?;
    }
    let mut buf: Vec<u8> = Vec::new();
    let mut req = reqwest::get("https://raw.githubusercontent.com/CBATeam/CBA_A3/master/addons/main/script_macros_common.hpp").unwrap();
    req.read_to_end(&mut buf)?;
    let mut out = File::create("include/x/cba/addons/main/script_macros_common.hpp")?;
    for c in &buf {
        out.write_all(&[*c])?;
    }
    Ok(())
}

pub fn create_addon(addon: &String, _p: &project::Project) -> Result<(), std::io::Error> {
    if !Path::new("addons").exists() {
        fs::create_dir("addons")?;
    }
    fs::create_dir(format!("addons/{}", addon))?;
    Ok(())
}
