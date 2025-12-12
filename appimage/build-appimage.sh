#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Get the project root directory (parent of script directory)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to project root
cd "$PROJECT_ROOT"

echo -e "${GREEN}Building AppImage for viewjson${NC}"

# Build release binary if not already built
if [ ! -f "target/release/viewjson" ]; then
    echo -e "${YELLOW}Building release binary...${NC}"
    cargo build --release
fi

# Create AppDir structure
APPDIR="viewjson.AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin"
mkdir -p "$APPDIR/usr/share/applications"
mkdir -p "$APPDIR/usr/share/icons/hicolor/256x256/apps"

# Copy binary
echo -e "${YELLOW}Copying binary...${NC}"
cp target/release/viewjson "$APPDIR/usr/bin/"

# Copy desktop file
echo -e "${YELLOW}Copying desktop file...${NC}"
cp appimage/viewjson.desktop "$APPDIR/usr/share/applications/"
cp appimage/viewjson.desktop "$APPDIR/"

# Create AppRun script (required for AppImage)
echo -e "${YELLOW}Creating AppRun script...${NC}"
cat > "$APPDIR/AppRun" << 'APPRUN_EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
exec "${HERE}/usr/bin/viewjson" "$@"
APPRUN_EOF
chmod +x "$APPDIR/AppRun"

# Create and install icon
echo -e "${YELLOW}Creating icon...${NC}"
ICON_PATH="$APPDIR/usr/share/icons/hicolor/256x256/apps/viewjson.png"
ICON_CREATED=false

if command -v convert &> /dev/null; then
    if convert -size 256x256 xc:blue -pointsize 72 -fill white -gravity center -annotate +0+0 "JSON" "$ICON_PATH" 2>/dev/null; then
        ICON_CREATED=true
        cp "$ICON_PATH" "$APPDIR/viewjson.png"
    fi
fi

# If icon creation failed, remove Icon line from desktop file to avoid errors
if [ "$ICON_CREATED" = false ]; then
    echo -e "${YELLOW}Warning: Could not create icon, removing Icon entry from desktop file...${NC}"
    sed -i '/^Icon=/d' "$APPDIR/viewjson.desktop"
    sed -i '/^Icon=/d' "$APPDIR/usr/share/applications/viewjson.desktop"
fi

# Download linuxdeploy if not present
LINUXDEPLOY="$PROJECT_ROOT/appimage/linuxdeploy-x86_64.AppImage"
if [ ! -f "$LINUXDEPLOY" ]; then
    echo -e "${YELLOW}Downloading linuxdeploy...${NC}"
    wget -q -O "$LINUXDEPLOY" https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage
    chmod +x "$LINUXDEPLOY"
fi

# Download GTK plugin if not present
GTK_PLUGIN="$PROJECT_ROOT/appimage/linuxdeploy-plugin-gtk.sh"
if [ ! -f "$GTK_PLUGIN" ]; then
    echo -e "${YELLOW}Downloading linuxdeploy GTK plugin...${NC}"
    wget -q -O "$GTK_PLUGIN" https://raw.githubusercontent.com/linuxdeploy/linuxdeploy-plugin-gtk/master/linuxdeploy-plugin-gtk.sh
    chmod +x "$GTK_PLUGIN"
fi

# Download appimagetool if not present
APPIMAGETOOL="$PROJECT_ROOT/appimage/appimagetool-x86_64.AppImage"
if [ ! -f "$APPIMAGETOOL" ]; then
    echo -e "${YELLOW}Downloading appimagetool...${NC}"
    wget -q -O "$APPIMAGETOOL" https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
    chmod +x "$APPIMAGETOOL"
fi

# Run linuxdeploy to bundle dependencies
echo -e "${YELLOW}Bundling dependencies with linuxdeploy...${NC}"
export LINUXDEPLOY="$LINUXDEPLOY"
# Use --icon-file if icon exists
ICON_ARG=""
if [ -f "$APPDIR/usr/share/icons/hicolor/256x256/apps/viewjson.png" ]; then
    ICON_ARG="--icon-file=$APPDIR/usr/share/icons/hicolor/256x256/apps/viewjson.png"
fi

"$LINUXDEPLOY" \
    --appdir "$APPDIR" \
    --executable "$APPDIR/usr/bin/viewjson" \
    --desktop-file "$APPDIR/viewjson.desktop" \
    $ICON_ARG \
    --plugin gtk \
    --output appimage || {
    echo -e "${RED}linuxdeploy failed, trying manual bundling...${NC}"
    # Fallback: try without plugin
    "$LINUXDEPLOY" \
        --appdir "$APPDIR" \
        --executable "$APPDIR/usr/bin/viewjson" \
        --desktop-file "$APPDIR/viewjson.desktop" \
        $ICON_ARG \
        --output appimage || true
}

# Ensure AppRun exists (linuxdeploy should create it, but ensure it's there)
if [ ! -f "$APPDIR/AppRun" ]; then
    echo -e "${YELLOW}Creating AppRun script (linuxdeploy didn't create it)...${NC}"
    cat > "$APPDIR/AppRun" << 'APPRUN_EOF'
#!/bin/bash
HERE="$(dirname "$(readlink -f "${0}")")"
exec "${HERE}/usr/bin/viewjson" "$@"
APPRUN_EOF
    chmod +x "$APPDIR/AppRun"
fi

# If linuxdeploy didn't create the AppImage, use appimagetool directly
if [ ! -f "viewjson-x86_64.AppImage" ]; then
    echo -e "${YELLOW}Creating AppImage with appimagetool...${NC}"
    "$APPIMAGETOOL" "$APPDIR" viewjson-x86_64.AppImage
fi

if [ -f "viewjson-x86_64.AppImage" ]; then
    echo -e "${GREEN}✓ AppImage created successfully: viewjson-x86_64.AppImage${NC}"
    chmod +x viewjson-x86_64.AppImage
    ls -lh viewjson-x86_64.AppImage
else
    echo -e "${RED}✗ Failed to create AppImage${NC}"
    exit 1
fi
