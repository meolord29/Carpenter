#!/bin/sh
# carpenter installer — fetches the `edge` release binary for this platform.
#
# End users:
#   curl -LsSf https://github.com/meolord29/Carpenter/releases/download/edge/install.sh | sh
#
# Test overrides (used by local verification, harmless to ignore):
#   CARPENTER_DOWNLOAD_BASE  base URL replacing the GitHub release dir
#   CARPENTER_INSTALL_DIR    destination dir (default ~/.local/bin)
set -eu

REPO="meolord29/Carpenter"
TAG="edge"
BASE="${CARPENTER_DOWNLOAD_BASE:-https://github.com/${REPO}/releases/download/${TAG}}"
INSTALL_DIR="${CARPENTER_INSTALL_DIR:-${HOME}/.local/bin}"

say() { printf 'carpenter: %s\n' "$1"; }
die() { printf 'carpenter: error: %s\n' "$1" >&2; exit 1; }

os=$(uname -s)
arch=$(uname -m)

case "$os:$arch" in
    Linux:x86_64)
        target=x86_64-unknown-linux-musl
        checksum="sha256sum" ;;
    Darwin:arm64)
        target=aarch64-apple-darwin
        checksum="shasum -a 256" ;;
    Darwin:x86_64)
        die "Intel Mac builds are not published; build from source: cargo xtask build --release" ;;
    *)
        die "unsupported platform: ${os} ${arch} (published: Linux x86_64, macOS Apple Silicon)" ;;
esac

tarball="carpenter-${target}.tar.gz"
case "$BASE" in
    file://*) ;;
    *) say "downloading ${tarball} from ${BASE}" ;;
esac

tmp=$(mktemp -d) || die "mktemp failed"
trap 'rm -rf "$tmp"' EXIT

curl -fsSL -o "${tmp}/${tarball}" "${BASE}/${tarball}" \
    || die "download failed: ${BASE}/${tarball}"
curl -fsSL -o "${tmp}/SHA256SUMS" "${BASE}/SHA256SUMS" \
    || die "download failed: ${BASE}/SHA256SUMS"

# shellcheck disable=SC2086
(cd "$tmp" && grep " ${tarball}\$" SHA256SUMS | $checksum -c - >/dev/null) \
    || die "checksum mismatch for ${tarball}"

tar -xzf "${tmp}/${tarball}" -C "$tmp" || die "extract failed: ${tarball}"
[ -f "${tmp}/carpenter" ] || die "tarball does not contain a 'carpenter' binary"

mkdir -p "$INSTALL_DIR" || die "cannot create ${INSTALL_DIR}"
install -m755 "${tmp}/carpenter" "${INSTALL_DIR}/carpenter" \
    || die "cannot write ${INSTALL_DIR}/carpenter (permissions?)"

case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
        # shellcheck disable=SC2016  # $PATH is literal text in the hint
        printf 'carpenter: note: %s is not on PATH; add it with: export PATH="%s:$PATH"\n' \
            "$INSTALL_DIR" "$INSTALL_DIR" >&2
        ;;
esac

if command -v opencode >/dev/null 2>&1; then
    say "opencode detected — registering carpenter skill"
    if "${INSTALL_DIR}/carpenter" register --app opencode >/dev/null 2>&1; then
        say "registered in opencode (skill + permission)"
    else
        printf 'carpenter: note: register failed; run manually: %s register --app opencode\n' \
            "${INSTALL_DIR}/carpenter" >&2
    fi
fi

say "installed $("${INSTALL_DIR}/carpenter" --version 2>/dev/null || echo carpenter)"
