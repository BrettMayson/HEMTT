let found = 0;
for child in HEMTT_RFS.list() {
    print(child);
    if (child.file_name().ends_with("docs")) {
        found += 1;
    }
    if (child.file_name().ends_with("releases")) {
        found += 1;
    }
    if (child.file_name().ends_with("addons")) {
        found += 1;
    }
    if (child.file_name().ends_with("include.txt")) {
        found += 1;
    }
}

if (found != 4) {
    fatal(":skull:");
}
