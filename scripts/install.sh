#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

# dictator installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/seuros/dictator/master/scripts/install.sh | bash

REPO="seuros/dictator"
DEFAULT_INSTALL_PREFIX="$HOME/.local/bin"

usage() {
  cat <<'EOF'
Usage: install.sh [options]

Options:
  --prefix <dir>    Install dictator into the specified directory
  --version <tag>   Install a specific git tag (with or without leading v)
  -h, --help        Show this help text

Environment overrides:
  DICTATOR_INSTALL_PREFIX       Install prefix override (same as --prefix)
  DICTATOR_INSTALL_VERSION      Version/tag to install (same as --version)

Default installation: ~/.local/bin (user-writable, no sudo needed)

The Dictator does not bow to system directories. Install locally.
EOF
}

log() { printf '%s\n' "$*"; }
info() { printf '==> %s\n' "$*"; }
warn() { printf 'WARN: %s\n' "$*" >&2; }
fail() { printf 'ERROR: %s\n' "$*" >&2; exit 1; }

expand_path() {
  local path="$1"
  case "$path" in
    "~")
      [ -n "${HOME:-}" ] || fail "HOME is not set; cannot expand ~"
      printf '%s\n' "$HOME"
      ;;
    "~/"*)
      [ -n "${HOME:-}" ] || fail "HOME is not set; cannot expand ~/"
      printf '%s/%s\n' "$HOME" "${path#~/}"
      ;;
    *)
      printf '%s\n' "$path"
      ;;
  esac
}

detect_http_client() {
  if command -v curl >/dev/null 2>&1; then
    HTTP_CLIENT="curl"
  elif command -v wget >/dev/null 2>&1; then
    HTTP_CLIENT="wget"
  else
    fail "curl or wget is required to download releases"
  fi
}

http_fetch() {
  local url="$1"
  if [ "$HTTP_CLIENT" = "curl" ]; then
    curl -fsSL --proto '=https' --tlsv1.2 "$url"
  else
    wget -qO- "$url"
  fi
}

http_download() {
  local url="$1"
  local dest="$2"
  if [ "$HTTP_CLIENT" = "curl" ]; then
    curl -fsSL --proto '=https' --tlsv1.2 -o "$dest" "$url"
  else
    wget -q -O "$dest" "$url"
  fi
}

parse_args() {
  REQUESTED_VERSION="${DICTATOR_INSTALL_VERSION:-}"
  PREFIX_OVERRIDE="${DICTATOR_INSTALL_PREFIX:-}"

  while [ "$#" -gt 0 ]; do
    case "$1" in
      --prefix)
        [ "$#" -ge 2 ] || fail "--prefix requires a directory argument"
        PREFIX_OVERRIDE="$2"
        shift
        ;;
      --version)
        [ "$#" -ge 2 ] || fail "--version requires a tag argument"
        REQUESTED_VERSION="$2"
        shift
        ;;
      -h|--help)
        usage
        exit 0
        ;;
      --system)
        fail "The Dictator does not bow to system directories. Use --prefix to install elsewhere."
        ;;
      *)
        fail "Unknown option: $1 (use --help for usage)"
        ;;
    esac
    shift
  done

  INSTALL_PREFIX="${PREFIX_OVERRIDE}"
  if [ -z "$INSTALL_PREFIX" ]; then
    INSTALL_PREFIX="$DEFAULT_INSTALL_PREFIX"
  fi

  INSTALL_PREFIX=$(expand_path "$INSTALL_PREFIX")
}

detect_platform() {
  local os uname_s arch uname_m
  uname_s=$(uname -s 2>/dev/null || true)
  uname_m=$(uname -m 2>/dev/null || true)

  case "$(echo "$uname_s" | tr '[:upper:]' '[:lower:]')" in
    linux)
      os="linux"
      ;;
    darwin)
      os="darwin"
      ;;
    freebsd)
      os="freebsd"
      ;;
    msys*|mingw*|cygwin*)
      os="windows"
      ;;
    *)
      fail "Unsupported operating system: ${uname_s}"
      ;;
  esac

  case "$uname_m" in
    x86_64|amd64)
      arch="x86_64"
      ;;
    aarch64|arm64)
      arch="aarch64"
      ;;
    *)
      fail "Unsupported architecture: ${uname_m}"
      ;;
  esac

  OS="$os"
  ARCH="$arch"
}

resolve_tag() {
  if [ -n "$REQUESTED_VERSION" ]; then
    TAG="v${REQUESTED_VERSION#v}"
    return
  fi

  if ! release_json=$(http_fetch "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null); then
    warn "GitHub API unavailable; falling back to release redirect"
  else
    TAG=$(printf '%s\n' "$release_json" | awk -F'"' '/"tag_name":/ {print $4; exit}')
  fi

  if [ -z "${TAG:-}" ]; then
    if [ "$HTTP_CLIENT" != "curl" ]; then
      fail "Unable to determine latest release (curl required for fallback). Please install curl or specify --version."
    fi
    TAG=$(curl -fsSL -o /dev/null -w '%{url_effective}' "https://github.com/${REPO}/releases/latest" 2>/dev/null || true)
    TAG="${TAG##*/}"
  fi

  if [ -z "${TAG:-}" ]; then
    fail "Could not determine latest release tag"
  fi
}

map_target() {
  local os="$1" arch="$2"
  case "${os}-${arch}" in
    linux-x86_64)
      echo "x86_64-unknown-linux-gnu"
      ;;
    linux-aarch64)
      fail "aarch64-linux not supported. The Dictator deemed it heretical."
      ;;
    darwin-x86_64)
      echo "x86_64-apple-darwin"
      ;;
    darwin-aarch64)
      echo "aarch64-apple-darwin"
      ;;
    freebsd-x86_64)
      echo "x86_64-unknown-freebsd"
      ;;
    windows-x86_64)
      echo "x86_64-pc-windows-msvc"
      ;;
    *)
      fail "No binary available for ${os}/${arch}"
      ;;
  esac
}

download_binary() {
  local tmp_dir tmp_bin download_url target_triple
  tmp_dir=$(mktemp -d 2>/dev/null || mktemp -d -t dictator-install)
  TMP_DIR="$tmp_dir"
  trap 'rm -rf "$TMP_DIR"' EXIT

  target_triple=$(map_target "$OS" "$ARCH")
  tmp_bin="${TMP_DIR}/dictator"

  # Determine file extension
  local archive_ext
  case "$OS" in
    windows) archive_ext="zip" ;;
    *) archive_ext="tar.gz" ;;
  esac

  BINARY_NAME="dictator-${TAG}-${target_triple}.${archive_ext}"
  download_url="https://github.com/${REPO}/releases/download/${TAG}/${BINARY_NAME}"

  info "Downloading dictator ${TAG} for ${OS}/${ARCH}"
  info "From: ${download_url}"

  if ! http_download "$download_url" "${TMP_DIR}/${BINARY_NAME}"; then
    fail "Failed to download ${download_url}. Check the tag/architecture and try again."
  fi

  # Extract binary
  if [ "$OS" = "windows" ]; then
    command -v unzip >/dev/null 2>&1 || fail "unzip required to extract Windows release"
    unzip -q "${TMP_DIR}/${BINARY_NAME}" -d "$TMP_DIR"
    if [ ! -f "${TMP_DIR}/dictator.exe" ]; then
      fail "Could not find dictator.exe in release archive"
    fi
    tmp_bin="${TMP_DIR}/dictator.exe"
  else
    command -v tar >/dev/null 2>&1 || fail "tar required to extract release"
    tar xzf "${TMP_DIR}/${BINARY_NAME}" -C "$TMP_DIR"
    if [ ! -f "${TMP_DIR}/dictator" ]; then
      fail "Could not find dictator binary in release archive"
    fi
  fi

  if [ ! -s "$tmp_bin" ]; then
    fail "Downloaded file is empty. The requested release may not provide ${BINARY_NAME}."
  fi

  chmod +x "$tmp_bin"
  DOWNLOADED_BIN="$tmp_bin"
}

ensure_install_prefix() {
  if ! mkdir -p "$INSTALL_PREFIX" 2>/dev/null; then
    fail "Cannot create install directory: ${INSTALL_PREFIX}. Check permissions."
  fi

  if [ ! -w "$INSTALL_PREFIX" ]; then
    fail "Install prefix ${INSTALL_PREFIX} is not writable. Choose a different --prefix or fix permissions."
  fi
}

install_binary() {
  local dest="${INSTALL_PREFIX}/dictator"
  mkdir -p "$INSTALL_PREFIX"
  install -m 0755 "$DOWNLOADED_BIN" "$dest"
  INSTALLED_PATH="$dest"
}

path_contains_dir() {
  local dir="$1"
  local old_ifs=$IFS entry
  IFS=":"
  for entry in ${PATH:-}; do
    if [ "$entry" = "$dir" ]; then
      IFS=$old_ifs
      return 0
    fi
  done
  IFS=$old_ifs
  return 1
}

print_completion() {
  info "Installed dictator to ${INSTALLED_PATH}"
  if [ -x "$INSTALLED_PATH" ]; then
    local version_output
    if version_output="$("$INSTALLED_PATH" --version 2>/dev/null)"; then
      log "$version_output"
    else
      warn "Installed binary could not report its version. Try running '${INSTALLED_PATH} --version'."
    fi
  fi

  if path_contains_dir "$INSTALL_PREFIX"; then
    log ""
    log "dictator is ready to use. Try 'dictator --help'."
  else
    warn "The directory ${INSTALL_PREFIX} is not on your PATH."
    if [ -n "${HOME:-}" ] && [[ "$INSTALL_PREFIX" == "$HOME/"* ]]; then
      local relative="${INSTALL_PREFIX#$HOME/}"
      cat <<EOF
Add the following line to your shell profile (e.g. ~/.bashrc or ~/.zshrc):
  export PATH="\$HOME/${relative}:\$PATH"

Then reload your shell or run: source ~/.bashrc
EOF
    else
      cat <<EOF
Add the following line to your shell profile (e.g. ~/.bashrc or ~/.zshrc):
  export PATH="${INSTALL_PREFIX}:\$PATH"

Then reload your shell or run: source ~/.bashrc
EOF
    fi
  fi

  log ""
  log "Quick start:"
  log "  dictator --help              Show available commands"
  log "  dictator lint .              Lint current directory"
  log ""
  log "See https://github.com/seuros/dictator for full documentation."
}

main() {
  command -v install >/dev/null 2>&1 || fail "'install' command not found. Please install coreutils/bsdinstall."
  detect_http_client
  parse_args "$@"
  detect_platform
  resolve_tag
  download_binary
  ensure_install_prefix
  install_binary
  print_completion
}

main "$@"
