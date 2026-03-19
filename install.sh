#!/bin/sh
set -eu

REPO="JamesGuthrie/unpm"
INSTALL_DIR="${HOME}/.local/bin"

main() {
    check_curl
    detect_platform
    resolve_version
    download_and_install
    print_success
}

check_curl() {
    if ! command -v curl >/dev/null 2>&1; then
        err "curl is required but not found. Please install curl and try again."
    fi
}

detect_platform() {
    case "$(uname -s)-$(uname -m)" in
        Linux-x86_64)       ARTIFACT="unpm-linux-x86_64" ;;
        Linux-aarch64)      ARTIFACT="unpm-linux-aarch64" ;;
        Darwin-x86_64)      ARTIFACT="unpm-darwin-x86_64" ;;
        Darwin-arm64)       ARTIFACT="unpm-darwin-aarch64" ;;
        *) err "Unsupported platform: $(uname -s) $(uname -m)" ;;
    esac
}

resolve_version() {
    printf "Fetching latest release tag... "
    TAG="$(curl --proto '=https' --tlsv1.2 -fSL \
        "https://api.github.com/repos/${REPO}/releases/latest" \
        2>/dev/null \
        | grep '"tag_name"' \
        | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/')"

    if [ -z "${TAG}" ]; then
        printf "\n"
        err "Failed to determine latest release version."
    fi
    printf "%s\n" "${TAG}"
}

download_and_install() {
    URL="https://github.com/${REPO}/releases/download/${TAG}/${ARTIFACT}"
    TMPFILE="$(mktemp)"
    trap 'rm -f "${TMPFILE}"' EXIT

    printf "Downloading %s... " "${ARTIFACT}"
    if ! curl --proto '=https' --tlsv1.2 -fSL -o "${TMPFILE}" "${URL}" 2>/dev/null; then
        printf "\n"
        err "Failed to download ${URL}"
    fi
    printf "done\n"

    mkdir -p "${INSTALL_DIR}"
    chmod +x "${TMPFILE}"
    mv "${TMPFILE}" "${INSTALL_DIR}/unpm"
}

print_success() {
    printf "\nInstalled unpm (%s) to %s/unpm\n" "${TAG}" "${INSTALL_DIR}"

    case ":${PATH}:" in
        *":${INSTALL_DIR}:"*)
            ;;
        *)
            printf "\n%s is not in your PATH. To add it:\n\n" "${INSTALL_DIR}"
            printf "  bash:  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc\n"
            printf "  zsh:   echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc\n"
            printf "  fish:  fish_add_path ~/.local/bin\n"
            printf "\nThen restart your shell or source the config file.\n"
            ;;
    esac
}

err() {
    printf "Error: %s\n" "$1" >&2
    exit 1
}

main
