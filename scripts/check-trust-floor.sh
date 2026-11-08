#!/bin/sh
# Fails when Cargo.lock carries a TLS trust crate below the trust floor:
#   webpki-roots / webpki-root-certs 1.x below 1.0.10
#   rustls-webpki 0.103.x below 0.103.16
# Older lines (webpki-roots 0.x, rustls-webpki 0.101/0.102) only warn.
#
# Any TOML layout cargo accepts is read: CRLF line endings, optional spaces
# around "=", either quote style, and keys in any order within a package.
set -eu

lock="${1:-$(dirname "$0")/../Cargo.lock}"

if [ ! -f "$lock" ]; then
    echo "trust floor: $lock not found" >&2
    exit 2
fi

awk '
function check(name, version,    v, major, minor, patch) {
    split(version, v, /[.+-]/)
    major = v[1] + 0; minor = v[2] + 0; patch = v[3] + 0
    if (name == "webpki-roots" || name == "webpki-root-certs") {
        if (major == 0) {
            printf "trust floor: warning: %s %s predates the 1.x line\n", name, version
        } else if (major == 1 && minor == 0 && patch < 10) {
            printf "trust floor: %s %s is below 1.0.10\n", name, version
            failed = 1
        }
    } else if (name == "rustls-webpki" && major == 0) {
        if (minor == 103 && patch < 16) {
            printf "trust floor: %s %s is below 0.103.16\n", name, version
            failed = 1
        } else if (minor < 103) {
            printf "trust floor: warning: %s %s predates the 0.103 line\n", name, version
        }
    }
}
# The string value of a `key = "value"` or `key = '\''value'\''` line.
function value(line,    q) {
    sub(/^[^=]*=[ \t]*/, "", line)
    q = substr(line, 1, 1)
    line = substr(line, 2)
    sub(q ".*$", "", line)
    return line
}
# Check the package entry just read, whatever order its keys came in.
function flush() {
    if (in_package && name != "" && version != "") check(name, version)
    in_package = 0; name = ""; version = ""
}
{ sub(/\r$/, "") }
/^[ \t]*\[/ {
    flush()
    if ($0 ~ /^[ \t]*\[\[[ \t]*package[ \t]*\]\]/) in_package = 1
    next
}
in_package && /^[ \t]*["'\'']?name["'\'']?[ \t]*=[ \t]*["'\'']/ { name = value($0) }
in_package && /^[ \t]*["'\'']?version["'\'']?[ \t]*=[ \t]*["'\'']/ { version = value($0) }
END {
    flush()
    if (failed) exit 1
    print "trust floor: ok"
}
' "$lock"
