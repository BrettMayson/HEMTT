let release = HEMTT_RFS.join("releases");
let src = release.join(HEMTT.project().prefix() + "-latest.zip");
let dst = release.join("@" + HEMTT.project().prefix() + ".zip");
debug("Moving zip to " + dst);
if !src.move(dst) {
    fatal("Failed to move " + src + " to " + dst);
}
