#!/bin/sh
set -eu

VERSION="${VERSION:-v0.1.0}"
REPO="yookibooki/web_search"

UNAME_S=$(uname -s)
UNAME_M=$(uname -m)

case "$UNAME_S" in
  Linux)
    INSTALL_DIR="${HOME}/.local/bin"
    case "$UNAME_M" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
      *) echo "unsupported arch: $UNAME_M" >&2; exit 1 ;;
    esac
    ;;
  Darwin)
    INSTALL_DIR="/usr/local/bin"
    case "$UNAME_M" in
      x86_64) TARGET="x86_64-apple-darwin" ;;
      arm64)  TARGET="aarch64-apple-darwin" ;;
      *) echo "unsupported arch: $UNAME_M" >&2; exit 1 ;;
    esac
    ;;
  *) echo "unsupported OS: $UNAME_S" >&2; exit 1 ;;
esac

BIN="web_search-${TARGET}"
URL="https://github.com/${REPO}/releases/download/${VERSION}/${BIN}"

mkdir -p "$INSTALL_DIR" 2>/dev/null || true

echo "downloading $BIN $VERSION..."
if command -v curl >/dev/null 2>&1; then
  curl -fsSL -o "${INSTALL_DIR}/web_search" "$URL" || {
    echo "failed to write to ${INSTALL_DIR}/web_search (permission denied)" >&2
    echo "try: sudo curl -fsSL -o ${INSTALL_DIR}/web_search \"$URL\" && sudo chmod +x ${INSTALL_DIR}/web_search" >&2
    exit 1
  }
elif command -v wget >/dev/null 2>&1; then
  wget -q -O "${INSTALL_DIR}/web_search" "$URL" || {
    echo "failed to write to ${INSTALL_DIR}/web_search (permission denied)" >&2
    echo "try: sudo wget -q -O ${INSTALL_DIR}/web_search \"$URL\" && sudo chmod +x ${INSTALL_DIR}/web_search" >&2
    exit 1
  }
else
  echo "need curl or wget" >&2
  exit 1
fi

chmod +x "${INSTALL_DIR}/web_search"
echo "installed to ${INSTALL_DIR}/web_search"

case ":${PATH}:" in
  *:${INSTALL_DIR}:*) ;;
  *) echo "note: ${INSTALL_DIR} is not in PATH" >&2 ;;
esac
