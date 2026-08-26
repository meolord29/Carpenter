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

# Register the skill into detected agent apps. Interactive (a TTY is attached):
# ask y/n per detected app, one by one. Non-interactive (CI, `curl | sh` under
# automation): auto-register every detected app so unattended lanes never hang.
# Prompts read /dev/tty — stdin carries the piped script under `curl | sh`.
interactive=no
if { [ -t 1 ] || [ -t 2 ]; } && [ -r /dev/tty ]; then
    interactive=yes
fi

ask_register() { # $1 = app label → y/n from the user
    printf 'carpenter: register the skill for %s? [y/N] ' "$1" >&2
    read -r answer </dev/tty || return 1
    case "$answer" in
        y|Y|yes|YES|Yes) return 0 ;;
        *) return 1 ;;
    esac
}

do_register() { # $1 = --app value, $2 = label for notes
    if "${INSTALL_DIR}/carpenter" register --app "$1" >/dev/null 2>&1; then
        say "registered in $2 (skill)"
    else
        printf 'carpenter: note: register failed; run manually: %s register --app %s\n' \
            "${INSTALL_DIR}/carpenter" "$1" >&2
    fi
}

detected=0
for app in opencode claude-code; do
    bin=$app
    [ "$app" = "claude-code" ] && bin=claude
    command -v "$bin" >/dev/null 2>&1 || continue
    detected=1
    if [ "$interactive" = "yes" ]; then
        if ask_register "$app"; then
            do_register "$app" "$app"
        else
            say "skipped $app (register later: carpenter register --app $app)"
        fi
    else
        say "$bin detected — registering carpenter skill (non-interactive)"
        do_register "$app" "$app"
    fi
done
if [ "$detected" = "0" ]; then
    printf 'carpenter: note: no agent app detected; register manually with: carpenter register --app opencode|claude-code\n' >&2
fi

say "installed $("${INSTALL_DIR}/carpenter" --version 2>/dev/null || echo carpenter)"
