let release = HEMTT_RFS.join("releases");
let src = release.join(HEMTT.project().prefix() + "-latest.zip");
let dst = release.join("@" + HEMTT.project().prefix() + ".zip");
print("Moving zip to " + dst);
if !src.move(dst) {
    print("Failed to move " + src + " to " + dst);
}
