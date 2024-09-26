use std::{fs::File, io::Write, process::Command};

use arma_bench::{Client, CompareRequest, CompareResult};
use hemtt_preprocessor::Processor;
use hemtt_sqf::{
    asc::{install, ASCConfig},
    parser::database::Database,
};
use hemtt_workspace::reporting::Processed;

pub fn compare(client: &Client, content: &str) -> Result<Vec<CompareResult>, String> {
    let workspace = hemtt_workspace::Workspace::builder()
        .memory()
        .finish(None, false, &hemtt_common::config::PDriveOption::Disallow)
        .expect("Failed to create workspace");
    let source_file = workspace
        .join("source.sqf")
        .expect("Failed to join source.sqf");
    source_file
        .create_file()
        .expect("Failed to create source.sqf")
        .write_all(content.as_bytes())
        .expect("Failed to write to source.sqf");

    let processed = Processor::run(&source_file).expect("Failed to process with Processor");
    let hemtt_blob = hemtt(&processed);
    let asc_blob = asc(&processed).expect("Failed to process with ASC");

    client.compare(vec![
        CompareRequest {
            id: 0,
            sqfc: false,
            content: content.bytes().collect(),
        },
        CompareRequest {
            id: 1,
            sqfc: true,
            content: hemtt_blob.0,
        },
        CompareRequest {
            id: 2,
            sqfc: true,
            content: hemtt_blob.1,
        },
        CompareRequest {
            id: 3,
            sqfc: true,
            content: asc_blob,
        },
    ])
}

fn hemtt(processed: &Processed) -> (Vec<u8>, Vec<u8>) {
    let parsed = match hemtt_sqf::parser::run(&Database::a3(false), processed) {
        Ok(sqf) => sqf,
        Err(hemtt_sqf::parser::ParserError::ParsingError(_)) => {
            panic!("failed to parse");
        }
        Err(e) => panic!("{e:?}"),
    };
    assert_ne!(parsed.content().len(), 0);
    (
        {
            let mut buffer = Vec::new();
            parsed
                .compile_to_writer(processed, &mut buffer)
                .expect("Failed to compile to writer");
            buffer
        },
        {
            let mut buffer = Vec::new();
            parsed
                .optimize()
                .compile_to_writer(processed, &mut buffer)
                .expect("Failed to compile to writer");
            buffer
        },
    )
}

fn asc(processed: &Processed) -> Result<Vec<u8>, String> {
    let asc_dir = std::env::temp_dir().join("hemtt_bench_asc");
    if !asc_dir.exists() {
        install(&asc_dir).expect("Failed to install ASC");
    }
    let source = asc_dir.join("source");
    let _ = std::fs::create_dir_all(&source);
    let file = source.join("source.sqf");
    {
        let mut file = File::create(&file).expect("Failed to create file");
        file.write_all(processed.as_str().as_bytes())
            .expect("Failed to write to file");
    }
    let output = asc_dir.join("output");
    let _ = std::fs::create_dir_all(&output);
    let mut config = ASCConfig::new();
    // config.add_input_dir(source.to_string_lossy().to_string());
    config.add_input_dir("source".to_string());
    config.set_output_dir(output.to_string_lossy().to_string());
    config.set_worker_threads(1);
    let mut f = File::create(asc_dir.join("sqfc.json")).expect("Failed to create sqfc.json");
    f.write_all(
        serde_json::to_string_pretty(&config)
            .expect("Failed to serialize config to json")
            .as_bytes(),
    )
    .expect("Failed to write to sqfc.json");
    let command = Command::new(asc_dir.join(if cfg!(target_os = "windows") {
        "asc.exe"
    } else {
        "asc"
    }))
    .current_dir(asc_dir)
    .output()
    .expect("Failed to run ASC");
    if command.status.success() {
        Ok(
            std::fs::read(output.join("source").join("source.sqfc"))
                .expect("Failed to read output"),
        )
    } else {
        println!("oof");
        println!(
            "o: {}",
            String::from_utf8(command.stdout.clone()).expect("stdout should be valid utf8")
        );
        println!(
            "e: {}",
            String::from_utf8(command.stderr.clone()).expect("stderr should be valid utf8")
        );
        Err(String::from_utf8(command.stderr).expect("stderr should be valid utf8"))
    }
}

pub fn display_compare(res: &[CompareResult]) {
    let r = res.first().expect("Failed to get 0");
    println!("Uncompiled - {}", r.id);
    println!("{} ms - {} iterations", r.time, r.iter);
    println!("Result: {}", r.ret);

    let r = res.get(1).expect("Failed to get 1");
    println!("\nHEMTT (No Optimizer) - {}", r.id);
    println!("{} ms - {} iterations", r.time, r.iter);
    println!("Result: {}", r.ret);

    let r = res.get(3).expect("Failed to get 2");
    println!("\nASC - {}", r.id);
    println!("{} ms - {} iterations", r.time, r.iter);
    println!("Result: {}", r.ret);

    let r = res.get(2).expect("Failed to get 1");
    println!("\nHEMTT (Optimizer) - {}", r.id);
    println!("{} ms - {} iterations", r.time, r.iter);
    println!("Result: {}", r.ret);
}
