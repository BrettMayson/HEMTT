pub fn is_ci() -> bool {
    // TODO: replace with crate if a decent one comes along
    let checks = vec![
        "CI",
        "APPVEYOR",
        "SYSTEM_TEAMFOUNDATIONCOLLECTIONURI",
        "bamboo_planKey",
        "BITBUCKET_COMMIT",
        "BITRISE_IO",
        "BUDDY_WORKSPACE_ID",
        "BUILDKITE",
        "CIRCLECI",
        "CIRRUS_CI",
        "CODEBUILD_BUILD_ARN",
        "DRONE",
        "DSARI",
        "GITLAB_CI",
        "GO_PIPELINE_LABEL",
        "HUDSON_URL",
        "MAGNUM",
        "NETLIFY_BUILD_BASE",
        "PULL_REQUEST",
        "NEVERCODE",
        "SAILCI",
        "SEMAPHORE",
        "SHIPPABLE",
        "TDDIUM",
        "STRIDER",
        "TEAMCITY_VERSION",
        "TRAVIS",
    ];
    for check in checks {
        if std::env::var(check).is_ok() {
            return true;
        }
    }
    false
}
