let include = HEMTT_OUT.join("include.txt");

if !include.exists() {
    fatal("{} does not exist", include);
}

if !include.is_file() {
    fatal("{} is not a file", include);
}

if include.is_dir() {
    fatal("{} is a directory", include);
}

if HEMTT.script("test") != 42 {
    fatal("HEMTT.script(\"test\") != 42");
}
