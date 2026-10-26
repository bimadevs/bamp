#!/usr/bin/env bash
set -euo pipefail

# BAMP ISO Builder Script
# Packages the BAMP application, portable templates, tools, and installer into an ISO image.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

VERSION="${1:-0.1.0}"
OUTPUT_DIR="${ROOT_DIR}/dist"
OUTPUT_ISO="${OUTPUT_DIR}/bamp-${VERSION}-linux-x86_64.iso"
STAGING_DIR="${OUTPUT_DIR}/iso-staging"

echo "=========================================="
echo " Building BAMP v${VERSION} ISO Distribution"
echo "=========================================="

mkdir -p "${OUTPUT_DIR}"
rm -rf "${STAGING_DIR}"
mkdir -p "${STAGING_DIR}"

# 1. Locate binary (prefer release, fallback to debug)
BINARY_PATH="${ROOT_DIR}/src-tauri/target/release/bamp"
if [[ ! -f "${BINARY_PATH}" ]]; then
    if [[ -f "${ROOT_DIR}/src-tauri/target/debug/bamp" ]]; then
        echo "[INFO] Release binary not found, using debug binary..."
        BINARY_PATH="${ROOT_DIR}/src-tauri/target/debug/bamp"
    else
        echo "[ERROR] No bamp binary found. Run 'pnpm tauri build' or 'cargo build --manifest-path src-tauri/Cargo.toml' first."
        exit 1
    fi
fi

echo "[1/5] Copying application core..."
cp "${BINARY_PATH}" "${STAGING_DIR}/bamp"
chmod +x "${STAGING_DIR}/bamp"

# 2. Copy bundled templates, www, and tooling
echo "[2/5] Copying stack runtime assets & templates..."
if [[ -d "${ROOT_DIR}/templates" ]]; then
    cp -r "${ROOT_DIR}/templates" "${STAGING_DIR}/templates"
fi

if [[ -d "${ROOT_DIR}/data/adminer" ]]; then
    mkdir -p "${STAGING_DIR}/data/adminer"
    cp -r "${ROOT_DIR}/data/adminer"/* "${STAGING_DIR}/data/adminer/" 2>/dev/null || true
fi

if [[ -d "${ROOT_DIR}/www" ]]; then
    cp -r "${ROOT_DIR}/www" "${STAGING_DIR}/www"
fi

# 3. Copy documentation and icons
echo "[3/5] Copying documentation & desktop shortcuts..."
cp "${ROOT_DIR}/README.md" "${STAGING_DIR}/README.md"
if [[ -f "${ROOT_DIR}/PANDUAN-PENGGUNA.md" ]]; then
    cp "${ROOT_DIR}/PANDUAN-PENGGUNA.md" "${STAGING_DIR}/PANDUAN-PENGGUNA.md"
fi

ICON_SRC="${ROOT_DIR}/src-tauri/icons/128x128.png"
if [[ -f "${ICON_SRC}" ]]; then
    cp "${ICON_SRC}" "${STAGING_DIR}/bamp.png"
fi

# 4. Copy packages (deb / appimage) if present
BUNDLE_DIR="${ROOT_DIR}/src-tauri/target/release/bundle"
if [[ -d "${BUNDLE_DIR}" ]]; then
    mkdir -p "${STAGING_DIR}/packages"
    find "${BUNDLE_DIR}" -type f \( -name "*.deb" -o -name "*.AppImage" -o -name "*.rpm" \) -exec cp {} "${STAGING_DIR}/packages/" \; 2>/dev/null || true
fi

# 5. Create launcher & installer scripts
cat << 'EOF' > "${STAGING_DIR}/run.sh"
#!/usr/bin/env bash
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export BAMP_PORTABLE_ROOT="$DIR"
exec "$DIR/bamp" "$@"
EOF
chmod +x "${STAGING_DIR}/run.sh"

cat << 'EOF' > "${STAGING_DIR}/install.sh"
#!/usr/bin/env bash
set -euo pipefail

echo "=========================================="
echo " Installing BAMP Local Development Stack  "
echo "=========================================="

DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_PREFIX="${HOME}/.local"
BIN_DIR="${INSTALL_PREFIX}/bin"
APP_DIR="${INSTALL_PREFIX}/share/bamp"
DESKTOP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/128x128/apps"

mkdir -p "${BIN_DIR}" "${APP_DIR}" "${DESKTOP_DIR}" "${ICON_DIR}"

echo "Installing files to ${APP_DIR}..."
cp -r "${DIR}/"* "${APP_DIR}/"
chmod +x "${APP_DIR}/bamp" "${APP_DIR}/run.sh"

echo "Creating launcher symlink in ${BIN_DIR}/bamp..."
ln -sf "${APP_DIR}/run.sh" "${BIN_DIR}/bamp"

if [[ -f "${DIR}/bamp.png" ]]; then
    cp "${DIR}/bamp.png" "${ICON_DIR}/bamp.png"
fi

echo "Creating desktop application entry..."
cat << DESKTOP > "${DESKTOP_DIR}/bamp.desktop"
[Desktop Entry]
Name=BAMP
Comment=Ultra-Lightweight Modern Local Development Stack Manager
Exec=${APP_DIR}/run.sh
Icon=bamp
Terminal=false
Type=Application
Categories=Development;WebDevelopment;
DESKTOP

chmod +x "${DESKTOP_DIR}/bamp.desktop"

echo "Installation complete!"
echo "You can launch BAMP from your application menu or run 'bamp' in your terminal."
EOF
chmod +x "${STAGING_DIR}/install.sh"

cat << 'EOF' > "${STAGING_DIR}/bamp.desktop"
[Desktop Entry]
Name=BAMP
Comment=Ultra-Lightweight Modern Local Development Stack Manager
Exec=./run.sh
Icon=bamp
Terminal=false
Type=Application
Categories=Development;WebDevelopment;
EOF

# 6. Generate ISO Image
echo "[4/5] Generating ISO-9660 filesystem image with xorriso..."
if command -v xorriso &>/dev/null; then
    xorriso -as mkisofs \
        -r \
        -J \
        -joliet-long \
        -V "BAMP_${VERSION}" \
        -o "${OUTPUT_ISO}" \
        "${STAGING_DIR}"
elif command -v genisoimage &>/dev/null; then
    genisoimage \
        -r \
        -J \
        -joliet-long \
        -V "BAMP_${VERSION}" \
        -o "${OUTPUT_ISO}" \
        "${STAGING_DIR}"
elif command -v mkisofs &>/dev/null; then
    mkisofs \
        -r \
        -J \
        -joliet-long \
        -V "BAMP_${VERSION}" \
        -o "${OUTPUT_ISO}" \
        "${STAGING_DIR}"
else
    echo "[ERROR] No ISO creation tool found (xorriso, genisoimage, or mkisofs)."
    exit 1
fi

# 7. Checksums and cleanup
echo "[5/5] Generating SHA256 checksum..."
cd "${OUTPUT_DIR}"
sha256sum "$(basename "${OUTPUT_ISO}")" > "$(basename "${OUTPUT_ISO}").sha256"
rm -rf "${STAGING_DIR}"

echo "=========================================="
echo " ISO Generated Successfully:"
echo " File:   ${OUTPUT_ISO}"
echo " Size:   $(du -h "${OUTPUT_ISO}" | cut -f1)"
echo " SHA256: $(cat "${OUTPUT_ISO}.sha256")"
echo "=========================================="
