#![allow(clippy::unwrap_used)]

use std::collections::HashSet;

use hemtt_p3d::{P3D, SearchCache};
use hemtt_workspace::WorkspacePath;

#[test]
fn ace_gunbag() {
    let buffer = fs_err::read("tests/ace_gunbag.p3d").unwrap();
    let mut read = std::io::Cursor::new(buffer.clone());
    let p3d = P3D::read(&mut read).unwrap();
    let mut textures = HashSet::new();
    for lod in &p3d.lods {
        for face in &lod.faces {
            textures.insert(face.texture.clone());
            assert_eq!(
                {
                    let mut buffer = Vec::new();
                    face.write(&mut buffer).unwrap();
                    hemtt_p3d::Face::read(&mut std::io::Cursor::new(buffer), false).unwrap()
                },
                *face
            );
        }
        assert_eq!(
            {
                let mut buffer = Vec::new();
                lod.write(&mut buffer).unwrap();
                hemtt_p3d::LOD::read(&mut std::io::Cursor::new(buffer)).unwrap()
            },
            *lod
        );
    }
    assert_eq!(textures.len(), 4);
    let mut out = Vec::new();
    p3d.write(&mut out).unwrap();
    assert_eq!(buffer, out);

    let missing = p3d
        .missing(
            &WorkspacePath::slim(&std::path::PathBuf::from(".vscode")).unwrap(),
            &SearchCache::new(),
        )
        .unwrap();
    assert_eq!(missing.0.len(), 1);
    assert_eq!(missing.1.len(), 2);
}

#[test]
fn kat_iv() {
    assert!(P3D::read(&mut fs_err::File::open("tests/kat_iv.p3d").unwrap()).is_ok());
}
