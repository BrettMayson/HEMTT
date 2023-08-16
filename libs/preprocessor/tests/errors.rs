// use std::{io::Read, path::PathBuf};

// use hemtt_preprocessor::{preprocess_file, Resolver};
// use vfs::PhysicalFS;

// const ROOT: &str = "tests/errors/";

// #[test]
// fn errors() {
//     // The output is slightly different on non-windows platforms
//     if !cfg!(windows) {
//         return;
//     }
//     for file in std::fs::read_dir(ROOT).unwrap() {
//         let file = file.unwrap();
//         if file.path().is_dir() {
//             println!(
//                 "errors {:?}",
//                 file.path().file_name().unwrap().to_str().unwrap()
//             );
//             let vfs =
//                 PhysicalFS::new(PathBuf::from(ROOT).join(file.path().file_name().unwrap())).into();
//             let resolver = Resolver::new(&vfs, Default::default());
//             let processed = preprocess_file(&vfs.join("source.hpp").unwrap(), &resolver);
//             match processed {
//                 Ok(config) => {
//                     panic!(
//                         "`{:?}` should have failed: {:#?}",
//                         file.path(),
//                         config.output()
//                     )
//                 }
//                 Err(e) => {
//                     let mut expected = Vec::new();
//                     std::fs::File::open(file.path().join("stderr.ansi"))
//                         .unwrap()
//                         .read_to_end(&mut expected)
//                         .unwrap();
//                     let error = e.get_code().unwrap().generate_report().unwrap();
//                     if expected.is_empty() {
//                         std::fs::write(
//                             file.path().join("stderr.ansi"),
//                             error.replace('\r', "").as_bytes(),
//                         )
//                         .unwrap();
//                     }
//                     assert_eq!(
//                         error.replace('\r', ""),
//                         String::from_utf8(expected).unwrap().replace('\r', "")
//                     );
//                 }
//             }
//         }
//     }
// }
