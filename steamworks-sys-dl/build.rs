#[cfg(feature = "rebuild-bindings")]
extern crate bindgen;

use std::env;
use std::fs::{self};
use std::path::{Path, PathBuf};

// Force rebuild
fn extract_functions_from_bindings(
    bindings_content: &str,
) -> Vec<(String, String, String, String)> {
    let mut functions = Vec::new();
    let mut in_extern_block = false;
    let mut current_block = String::new();
    let mut block_count = 0;
    let mut debug_log = format!("Bindings content length: {}\n", bindings_content.len());

    for line in bindings_content.lines() {
        if line.trim().starts_with("extern \"C\"") && line.trim().ends_with("{") {
            debug_log.push_str(&format!("Found extern block start at: {}\n", line.trim()));
            in_extern_block = true;
            current_block.clear();
            continue;
        }

        if in_extern_block {
            if line.trim() == "}" {
                in_extern_block = false;
                block_count += 1;
                // Extract function from the block
                if let Some(func) = extract_function_from_block(&current_block) {
                    debug_log.push_str(&format!("Extracted function {}: {}\n", func.0, func.1));
                    functions.push(func);
                } else {
                    debug_log.push_str(&format!(
                        "Failed to extract from block (first 100 chars): {}...\n",
                        &current_block[..std::cmp::min(100, current_block.len())]
                    ));
                }
                current_block.clear();
            } else {
                current_block.push_str(line);
                current_block.push('\n');
            }
        }
    }

    debug_log.push_str(&format!(
        "Found {} functions from {} extern blocks\n",
        functions.len(),
        block_count
    ));

    // Write debug log to file
    let out_dir = env::var("OUT_DIR").unwrap_or_default();
    if !out_dir.is_empty() {
        let _ = fs::write(format!("{}/DEBUG_extract.txt", out_dir), &debug_log);
    }

    functions
}

fn extract_function_from_block(block: &str) -> Option<(String, String, String, String)> {
    // Extract #[link_name] attribute and function signature
    // Returns: (rust_name, native_symbol_name, params, return_type)
    let mut sig = String::new();
    let mut in_function = false;
    let mut link_name: Option<String> = None;

    for line in block.lines() {
        let trimmed = line.trim();

        // Look for #[link_name = "..."] attribute
        if trimmed.starts_with("#[link_name") {
            if let Some(start) = trimmed.find('"') {
                if let Some(end) = trimmed.rfind('"') {
                    if start < end {
                        link_name = Some(trimmed[start + 1..end].to_string());
                    }
                }
            }
            continue;
        }

        // Skip other attributes
        if trimmed.starts_with("#[") {
            continue;
        }

        // Look for "pub fn"
        if trimmed.starts_with("pub fn") {
            in_function = true;
        }

        if in_function {
            sig.push_str(trimmed);
            sig.push(' ');

            // Check if this line ends the function signature (has ;)
            if trimmed.ends_with(";") {
                // Parse the signature - trim it first to remove trailing spaces
                if let Some((name, params, ret_type)) = parse_function_signature(sig.trim()) {
                    let native_name = link_name.unwrap_or_else(|| name.clone());
                    return Some((name, native_name, params, ret_type));
                }
                return None;
            }
        }
    }

    None
}

fn parse_function_signature(sig: &str) -> Option<(String, String, String)> {
    // Parse: pub fn NAME(PARAMS) -> RETURN_TYPE;
    if !sig.starts_with("pub fn") || !sig.ends_with(";") {
        return None;
    }

    let sig_without_pub = sig.strip_prefix("pub fn")?.trim();
    let sig_without_trailing = sig_without_pub.strip_suffix(";")?;

    // Find the opening paren
    let paren_idx = sig_without_trailing.find('(')?;
    let name = sig_without_trailing[..paren_idx].trim().to_string();

    // Find the matching closing paren - need to handle nested parens/brackets
    let mut paren_depth: i32 = 0;
    let mut bracket_depth: i32 = 0;
    let mut angle_depth: i32 = 0;
    let mut close_paren_idx = None;

    for (i, c) in sig_without_trailing[paren_idx..].char_indices() {
        match c {
            '(' => paren_depth += 1,
            ')' => {
                paren_depth -= 1;
                if paren_depth == 0 {
                    close_paren_idx = Some(paren_idx + i);
                    break;
                }
            }
            '[' => bracket_depth += 1,
            ']' => bracket_depth = (bracket_depth - 1).max(0),
            '<' if !is_lifetime_marker(&sig_without_trailing[paren_idx + i..]) => {
                angle_depth += 1;
            }
            '>' if angle_depth > 0 => angle_depth -= 1,
            _ => {}
        }
    }

    let close_paren_idx = close_paren_idx?;
    let params = sig_without_trailing[paren_idx + 1..close_paren_idx]
        .trim()
        .to_string();

    let after_paren = &sig_without_trailing[close_paren_idx + 1..].trim();
    let return_type = if let Some(ret) = after_paren.strip_prefix("->") {
        ret.trim().to_string()
    } else {
        "()".to_string()
    };

    Some((name, params, return_type))
}

fn is_lifetime_marker(s: &str) -> bool {
    // Simple heuristic: if it's followed by 'a-z (like 'a, 'b), it's likely a lifetime
    s.starts_with("'")
        && s.len() > 1
        && s.chars().nth(1).map(|c| c.is_alphabetic()).unwrap_or(false)
}

fn generate_wrappers_and_filtered_bindings(
    bindings_path: &Path,
    out_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let bindings_content = fs::read_to_string(bindings_path)?;

    // Debug: write first part of bindings to see structure
    if let Ok(first_1000) = fs::write(
        out_path.join("DEBUG_bindings_head.txt"),
        &bindings_content[..std::cmp::min(2000, bindings_content.len())],
    ) {
        eprintln!("DEBUG: Wrote bindings head to DEBUG_bindings_head.txt");
    }

    // Extract function signatures with symbol names
    let functions = extract_functions_from_bindings(&bindings_content);

    eprintln!("Found {} functions to wrap", functions.len());

    // List of critical symbols that must load successfully
    let required_symbols = [
        "SteamAPI_InitFlat",
        "SteamAPI_Shutdown",
        "SteamAPI_ManualDispatch_Init",
        "SteamAPI_ManualDispatch_RunFrame",
        "SteamAPI_ManualDispatch_GetNextCallback",
        "SteamAPI_ManualDispatch_FreeLastCallback",
    ];

    // Generate wrappers
    let mut wrappers = String::new();
    wrappers.push_str("// AUTO-GENERATED STEAM API WRAPPERS - DO NOT EDIT\n");
    wrappers.push_str("// This module provides runtime dynamic loading of Steam API functions.\n");
    wrappers.push_str("// Symbol loading MUST succeed for critical functions.\n\n");

    // Generate type aliases
    for (rust_name, _native_name, params, ret_type) in &functions {
        let fn_ptr = format!("unsafe extern \"C\" fn({}) -> {}", params, ret_type);
        wrappers.push_str(&format!("type FnPtr_{} = {};\n", rust_name, fn_ptr));
    }
    wrappers.push_str("\n");

    // Global state
    wrappers.push_str("/// Storage for function pointers\n");
    wrappers.push_str("struct SteamApiFunctions {\n");
    for (rust_name, _native_name, _params, _ret_type) in &functions {
        wrappers.push_str(&format!(
            "    {}: Option<FnPtr_{}>,\n",
            rust_name, rust_name
        ));
    }
    wrappers.push_str("}\n\n");

    wrappers.push_str("static STEAM_API: std::sync::OnceLock<SteamApiFunctions> = std::sync::OnceLock::new();\n\n");

    // Generate platform-specific initialization functions with detailed error reporting
    wrappers
        .push_str("/// Initialize Steam API function pointers from a dynamically loaded library\n");
    wrappers.push_str("/// Returns detailed error information if symbol loading fails\n");
    wrappers.push_str("#[cfg(target_os = \"windows\")]\n");
    wrappers.push_str("pub(crate) unsafe fn initialize_steam_api(\n");
    wrappers.push_str("    lib: &libloading::Library,\n");
    wrappers.push_str(") -> Result<(), Box<dyn std::error::Error>> {\n");
    wrappers.push_str("    let mut api = SteamApiFunctions {\n");

    for (rust_name, native_name, _params, _ret_type) in &functions {
        let is_required = required_symbols.contains(&native_name.as_str());
        if is_required {
            wrappers.push_str(&format!(
                "        {}: Some(*lib.get::<FnPtr_{}>(\"{}\" .as_bytes())\n",
                rust_name, rust_name, native_name
            ));
            wrappers.push_str(&format!(
                "            .map_err(|e| format!(\"Failed to load required symbol '{}': {{}}\", e))?),\n",
                native_name
            ));
        } else {
            wrappers.push_str(&format!(
                "        {}: lib.get::<FnPtr_{}>(\"{}\" .as_bytes()).ok().map(|s| *s),\n",
                rust_name, rust_name, native_name
            ));
        }
    }

    wrappers.push_str("    };\n");
    wrappers.push_str("    let _ = STEAM_API.set(api);\n");
    wrappers.push_str("    Ok(())\n");
    wrappers.push_str("}\n\n");

    // Unix version (Linux)
    wrappers.push_str("#[cfg(target_os = \"linux\")]\n");
    wrappers.push_str("pub(crate) unsafe fn initialize_steam_api(\n");
    wrappers.push_str("    lib: &libloading::os::unix::Library,\n");
    wrappers.push_str(") -> Result<(), Box<dyn std::error::Error>> {\n");
    wrappers.push_str("    let mut api = SteamApiFunctions {\n");

    for (rust_name, native_name, _params, _ret_type) in &functions {
        let is_required = required_symbols.contains(&native_name.as_str());
        if is_required {
            wrappers.push_str(&format!(
                "        {}: Some(*lib.get::<FnPtr_{}>(\"{}\" .as_bytes())\n",
                rust_name, rust_name, native_name
            ));
            wrappers.push_str(&format!(
                "            .map_err(|e| format!(\"Failed to load required symbol '{}': {{}}\", e))?),\n",
                native_name
            ));
        } else {
            wrappers.push_str(&format!(
                "        {}: lib.get::<FnPtr_{}>(\"{}\" .as_bytes()).ok().map(|s| *s),\n",
                rust_name, rust_name, native_name
            ));
        }
    }

    wrappers.push_str("    };\n");
    wrappers.push_str("    let _ = STEAM_API.set(api);\n");
    wrappers.push_str("    Ok(())\n");
    wrappers.push_str("}\n\n");

    // macOS version
    wrappers.push_str("#[cfg(target_os = \"macos\")]\n");
    wrappers.push_str("pub(crate) unsafe fn initialize_steam_api(\n");
    wrappers.push_str("    lib: &libloading::os::unix::Library,\n");
    wrappers.push_str(") -> Result<(), Box<dyn std::error::Error>> {\n");
    wrappers.push_str("    let mut api = SteamApiFunctions {\n");

    for (rust_name, native_name, _params, _ret_type) in &functions {
        let is_required = required_symbols.contains(&native_name.as_str());
        if is_required {
            wrappers.push_str(&format!(
                "        {}: Some(*lib.get::<FnPtr_{}>(\"{}\" .as_bytes())\n",
                rust_name, rust_name, native_name
            ));
            wrappers.push_str(&format!(
                "            .map_err(|e| format!(\"Failed to load required symbol '{}': {{}}\", e))?),\n",
                native_name
            ));
        } else {
            wrappers.push_str(&format!(
                "        {}: lib.get::<FnPtr_{}>(\"{}\" .as_bytes()).ok().map(|s| *s),\n",
                rust_name, rust_name, native_name
            ));
        }
    }

    wrappers.push_str("    };\n");
    wrappers.push_str("    let _ = STEAM_API.set(api);\n");
    wrappers.push_str("    Ok(())\n");
    wrappers.push_str("}\n\n");

    // Generate wrapper functions
    for (rust_name, _native_name, params, ret_type) in &functions {
        wrappers.push_str(&format!("/// Steam API wrapper: {}\n", rust_name));
        wrappers.push_str(&format!("pub unsafe fn {}(", rust_name));
        if !params.is_empty() {
            wrappers.push_str(params);
        }
        wrappers.push_str(&format!(") -> {} {{\n", ret_type));

        // Extract parameter names
        let param_names: Vec<String> = params
            .split(',')
            .filter_map(|p| {
                let trimmed = p.trim();
                if trimmed.is_empty() {
                    return None;
                }
                trimmed.split(':').next().map(|s| s.trim().to_string())
            })
            .collect();

        // Body
        if ret_type.trim() == "()" {
            wrappers.push_str("    if let Some(api) = STEAM_API.get() {\n");
            wrappers.push_str(&format!("        if let Some(f) = api.{} {{\n", rust_name));
            wrappers.push_str(&format!("            f({})\n", param_names.join(", ")));
            wrappers.push_str("        }\n");
            wrappers.push_str("    }\n");
        } else {
            wrappers.push_str("    STEAM_API\n");
            wrappers.push_str("        .get()\n");
            wrappers.push_str("        .and_then(|api| api.");
            wrappers.push_str(&format!("{}", rust_name));
            wrappers.push_str(")\n");
            wrappers.push_str(&format!(
                "        .map(|f| f({}))\n",
                param_names.join(", ")
            ));
            wrappers
                .push_str("        .unwrap_or_else(|| panic!(\"Steam API not initialized\"))\n");
        }

        wrappers.push_str("}\n\n");
    }

    let wrappers_path = out_path.join("steam_api_wrappers.rs");
    fs::write(wrappers_path, wrappers)?;
    eprintln!(
        "Generated wrappers to {}",
        out_path.join("steam_api_wrappers.rs").display()
    );

    // Filter bindings
    let filtered_bindings = filter_bindings_remove_externs(&bindings_content);
    let filtered_path = out_path.join("bindings_no_externs.rs");
    fs::write(filtered_path, filtered_bindings)?;
    eprintln!(
        "Generated filtered bindings to {}",
        out_path.join("bindings_no_externs.rs").display()
    );

    Ok(())
}

fn filter_bindings_remove_externs(content: &str) -> String {
    // Instead of removing extern blocks entirely, replace them with wrapper declarations
    // that will be defined in steam_api_wrappers.rs
    let mut output = String::new();
    let mut skip_extern_block = false;
    let mut brace_depth = 0;

    for line in content.lines() {
        // Check if we're entering an extern block
        if line.trim().starts_with("extern \"C\"") && line.trim().ends_with("{") {
            skip_extern_block = true;
            brace_depth = 1;
            continue;
        }

        // Track braces while skipping
        if skip_extern_block {
            brace_depth += line.matches('{').count();
            brace_depth = brace_depth.saturating_sub(line.matches('}').count());

            if brace_depth == 0 {
                skip_extern_block = false;
            }
            continue;
        }

        // Keep everything else (types, constants, impl blocks, etc.)
        output.push_str(line);
        output.push('\n');
    }

    output
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let sdk_loc = if let Ok(sdk_loc) = env::var("STEAM_SDK_LOCATION") {
        Path::new(&sdk_loc).to_path_buf()
    } else {
        let mut path = PathBuf::new();
        path.push(env::var("CARGO_MANIFEST_DIR").unwrap());
        path.push("lib");
        path.push("steam");
        path
    };
    println!("cargo:rerun-if-env-changed=STEAM_SDK_LOCATION");

    let triple = env::var("TARGET").unwrap();
    let mut lib = "steam_api";
    let mut link_path = sdk_loc.join("redistributable_bin");
    if triple.contains("windows") {
        if !triple.contains("i686") {
            lib = "steam_api64";
            link_path.push("win64");
        }
    } else if triple.contains("linux") {
        if triple.contains("i686") {
            link_path.push("linux32");
        } else if triple.contains("aarch64") {
            link_path.push("linuxarm64");
        } else {
            link_path.push("linux64");
        }
    } else if triple.contains("darwin") {
        link_path.push("osx");
    } else {
        panic!("Unsupported OS");
    };

    if triple.contains("windows") {
        let dll_file = format!("{}.dll", lib);
        let lib_file = format!("{}.lib", lib);
        fs::copy(link_path.join(&dll_file), out_path.join(dll_file))?;
        fs::copy(link_path.join(&lib_file), out_path.join(lib_file))?;
    } else if triple.contains("darwin") {
        fs::copy(
            link_path.join("libsteam_api.dylib"),
            out_path.join("libsteam_api.dylib"),
        )?;
    } else if triple.contains("linux") {
        fs::copy(
            link_path.join("libsteam_api.so"),
            out_path.join("libsteam_api.so"),
        )?;
    }

    println!("cargo:rustc-link-search={}", out_path.display());
    // REMOVED: println!("cargo:rustc-link-lib=dylib={}", lib);

    let target_os = if triple.contains("windows") {
        "windows"
    } else if triple.contains("darwin") {
        "macos"
    } else if triple.contains("linux") {
        if triple.contains("aarch64") {
            "linuxarm"
        } else {
            "linux"
        }
    } else {
        panic!("Unsupported OS");
    };

    #[cfg(feature = "rebuild-bindings")]
    {
        let binding_path = Path::new(&format!("src/{}_bindings.rs", target_os)).to_owned();
        let bindings = bindgen::Builder::default()
            .header(
                sdk_loc
                    .join("public/steam/steam_api_flat.h")
                    .to_string_lossy(),
            )
            .header(
                sdk_loc
                    .join("public/steam/steam_gameserver.h")
                    .to_string_lossy(),
            )
            .clang_arg("-xc++")
            .clang_arg("-std=c++11")
            .clang_arg(format!("-I{}", sdk_loc.join("public").display()))
            .allowlist_function("Steam.*")
            .allowlist_var(".*")
            .allowlist_type(".*")
            .default_enum_style(bindgen::EnumVariation::Rust {
                non_exhaustive: true,
            })
            .bitfield_enum("EMarketNotAllowedReasonFlags")
            .bitfield_enum("EBetaBranchFlags")
            .bitfield_enum("EFriendFlags")
            .bitfield_enum("EPersonaChange")
            .bitfield_enum("ERemoteStoragePlatform")
            .bitfield_enum("EChatSteamIDInstanceFlags")
            .bitfield_enum("ESteamItemFlags")
            .bitfield_enum("EOverlayToStoreFlag")
            .bitfield_enum("EChatSteamIDInstanceFlags")
            .generate()
            .expect("Unable to generate bindings");

        bindings
            .write_to_file(&binding_path)
            .expect("Couldn't write bindings!");
    }

    // Generate wrappers from bindgen output
    if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let binding_path = Path::new(&manifest_dir)
            .join("src")
            .join(format!("{}_bindings.rs", target_os));
        if binding_path.exists() {
            if let Err(e) = generate_wrappers_and_filtered_bindings(&binding_path, &out_path) {
                eprintln!("Warning: Failed to generate wrappers: {}", e);
            }
        }
    }

    Ok(())
}
