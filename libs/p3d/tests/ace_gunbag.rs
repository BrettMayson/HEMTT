use std::collections::HashSet;

use hemtt_p3d::P3D;

#[test]
fn ace_gunbag() {
    let p3d = P3D::read(&mut std::fs::File::open("tests/ace_gunbag.p3d").unwrap()).unwrap();
    let mut textures = HashSet::new();
    for lod in &p3d.lods {
        for face in &lod.faces {
            textures.insert(face.texture.clone());
        }
    }
    assert_eq!(textures.len(), 4);
}
