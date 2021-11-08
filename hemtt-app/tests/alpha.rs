use std::fs::File;

use hemtt::Project;

use semver::Version;

#[test]
fn project_info() {
    std::env::set_current_dir("./tests/mod_alpha").unwrap();
    let root = std::env::current_dir().unwrap();
    let project = Project::read().unwrap();
    assert_eq!(project.name(), "CBA Base Template");
    assert_eq!(project.author(), "CBA Base Template");
    assert_eq!(project.prefix(), "test");
    assert_eq!(project.mainprefix(), "z");
    assert_eq!(project.template(), "cba");
    assert_eq!(project.version(), &Version::new(0, 1, 0));
    assert_eq!(project.authority().unwrap(), "cba_base_template-0.1.0");
    assert_eq!(project.sig_version(), 3);

    assert_eq!(hemtt::project::get_all_addons().unwrap().len(), 1);

    assert_eq!(Project::find_root().unwrap(), root);
    std::env::set_current_dir("./addons").unwrap();
    assert_eq!(Project::find_root().unwrap(), root);

    hemtt_app::execute(
        &[
            "hemtt-app".to_string(),
            "project".to_string(),
            "version".to_string(),
        ],
        true,
    )
    .unwrap();
    hemtt_app::execute(&["hemtt-app".to_string(), "build".to_string()], false).unwrap();

    hemtt_pbo::tests::sync::pbo(
        File::open("addons/test_main.pbo").unwrap(),
        5,
        true,
        3,
        "0.1.0",
        "z\\test\\addons\\main",
        // TODO fix CLRF issues
        if cfg!(windows) {
            vec![
                185, 198, 176, 91, 142, 39, 245, 35, 253, 167, 56, 163, 131, 235, 252, 190, 245,
                179, 29, 207,
            ]
        } else {
            vec![
                141, 114, 181, 121, 251, 194, 7, 240, 49, 229, 215, 100, 55, 187, 237, 83, 145,
                152, 104, 57,
            ]
        },
    );
}
