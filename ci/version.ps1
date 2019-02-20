If ($ENV:APPVEYOR_REPO_TAG -eq "false" ) {
    $toml = Get-Content Cargo.toml -Raw
    [regex]$pattern = "version\s?=\s?`"(.+?)`""
    Set-Content -Path Cargo.toml -Value (
        $pattern.replace($toml, ("version = `"`$1-" + $ENV:APPVEYOR_REPO_COMMIT + "`""), 1)
    )
}
