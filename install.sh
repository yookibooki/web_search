#!/bin/sh
set -eu

VERSION="${VERSION:-v0.1.0}"
HTML2MARKDOWN_VERSION="${HTML2MARKDOWN_VERSION:-v2.5.2}"
HTML2MARKDOWN_VERSION_NO_V="${HTML2MARKDOWN_VERSION#v}"
REPO="yookibooki/web_search"
HTML_REPO="JohannesKaufmann/html-to-markdown"

UNAME_S=$(uname -s)
UNAME_M=$(uname -m)

case "$UNAME_S" in
  Linux)
    INSTALL_DIR="${HOME}/.local/bin"
    case "$UNAME_M" in
      x86_64) TARGET="x86_64-unknown-linux-gnu"; HTML_TARGET="Linux_x86_64" ;;
      aarch64|arm64) TARGET="aarch64-unknown-linux-gnu"; HTML_TARGET="Linux_arm64" ;;
      i686|i386) TARGET="i686-unknown-linux-gnu"; HTML_TARGET="Linux_i386" ;;
      *) echo "unsupported arch: $UNAME_M" >&2; exit 1 ;;
    esac
    ;;
  Darwin)
    INSTALL_DIR="/usr/local/bin"
    case "$UNAME_M" in
      x86_64) TARGET="x86_64-apple-darwin"; HTML_TARGET="Darwin_x86_64" ;;
      arm64)  TARGET="aarch64-apple-darwin"; HTML_TARGET="Darwin_arm64" ;;
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

HTML_BIN="html-to-markdown_${HTML2MARKDOWN_VERSION_NO_V}_${HTML_TARGET}.tar.gz"
HTML_URL="https://github.com/${HTML_REPO}/releases/download/${HTML2MARKDOWN_VERSION}/${HTML_BIN}"

echo "downloading html2markdown ${HTML2MARKDOWN_VERSION}..."
TMPDIR=$(mktemp -d 2>/dev/null || mktemp -d -t html2markdown)
if command -v curl >/dev/null 2>&1; then
  curl -fsSL -o "${TMPDIR}/${HTML_BIN}" "$HTML_URL" || {
    echo "failed to download html2markdown" >&2
    rm -rf "$TMPDIR"
    exit 1
  }
elif command -v wget >/dev/null 2>&1; then
  wget -q -O "${TMPDIR}/${HTML_BIN}" "$HTML_URL" || {
    echo "failed to download html2markdown" >&2
    rm -rf "$TMPDIR"
    exit 1
  }
else
  echo "need curl or wget" >&2
  exit 1
fi

tar -xzf "${TMPDIR}/${HTML_BIN}" -C "$TMPDIR"
mv "$TMPDIR/html2markdown" "${INSTALL_DIR}/html2markdown"
chmod +x "${INSTALL_DIR}/html2markdown"
rm -rf "$TMPDIR"
echo "installed to ${INSTALL_DIR}/html2markdown"

case ":${PATH}:" in
  *:${INSTALL_DIR}:*) ;;
  *) echo "note: ${INSTALL_DIR} is not in PATH" >&2 ;;
esac
