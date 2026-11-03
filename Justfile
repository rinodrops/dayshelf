set windows-shell := ["sh", "-cu"]

app_name  := "DayShelf"
exe_name  := "dayshelf"
bundle_id := "jp.emotiongraphics.dayshelf"
min_macos := "13.0"
version   := `awk -F'"' '/^version *=/{print $2; exit}' Cargo.toml`
pkg_name  := "DayShelf"

rust_target_arm64 := "aarch64-apple-darwin"
rust_target_x86   := "x86_64-apple-darwin"
win_target        := "x86_64-pc-windows-gnu"
win_target_dir    := "/tmp/dayshelf-win"

settings_dir    := env_var_or_default("SETTINGS_DIR", "../settings")
settings_schema := justfile_directory() + "/schema.toml"

default: help

help:
    @just --list

dev: _require-settings _settings-host
    cargo build
    just _copy-host-settings target/debug
    @echo "Debug build: target/debug/{{exe_name}}"

# ---------------------------------------------------------------------------
# macOS
# ---------------------------------------------------------------------------

darwin-build: darwin-build-arm64 darwin-build-x86_64

darwin-build-arm64: _require-settings _settings-arm64
    just _darwin-bundle darwin-arm64 {{rust_target_arm64}}
    @echo "App bundle: dist/darwin-arm64/{{app_name}}.app"

darwin-build-x86_64: _require-settings _settings-x86_64
    just _darwin-bundle darwin-x86_64 {{rust_target_x86}}
    @echo "App bundle: dist/darwin-x86_64/{{app_name}}.app"

[macos]
install: darwin-build-arm64
    rm -rf "/Applications/{{app_name}}.app"
    cp -r "dist/darwin-arm64/{{app_name}}.app" "/Applications/"

# ---------------------------------------------------------------------------
# Windows
# ---------------------------------------------------------------------------

win-build: _require-settings _settings-win
    CARGO_TARGET_DIR="{{win_target_dir}}" cargo build --release --target {{win_target}}
    mkdir -p dist/windows-x86_64
    cp "{{win_target_dir}}/{{win_target}}/release/{{exe_name}}.exe" \
        "dist/windows-x86_64/{{exe_name}}.exe"
    cp "{{settings_dir}}/dist/settings/windows-x86_64/Settings.exe" \
        "dist/windows-x86_64/settings.exe"
    @echo "Windows build: dist/windows-x86_64"

win-zip: win-build
    #!/usr/bin/env bash
    set -euo pipefail
    ZIP="dist/{{pkg_name}}-v{{version}}-windows-x86_64.zip"
    rm -f "${ZIP}"
    cd dist/windows-x86_64 && zip "../$(basename "${ZIP}")" "{{exe_name}}.exe" settings.exe
    echo "Package: ${ZIP}"

[windows]
install: win-build
    mkdir -p "$LOCALAPPDATA/Programs/{{exe_name}}"
    cp "dist/windows-x86_64/{{exe_name}}.exe" "$LOCALAPPDATA/Programs/{{exe_name}}/"
    cp "dist/windows-x86_64/settings.exe" "$LOCALAPPDATA/Programs/{{exe_name}}/"

# ---------------------------------------------------------------------------
# Linux
# ---------------------------------------------------------------------------

[linux]
linux-build: _require-settings _settings-linux
    cargo build --release
    mkdir -p dist/linux-x86_64
    cp "target/release/{{exe_name}}" "dist/linux-x86_64/{{exe_name}}"
    cp "{{settings_dir}}/dist/settings/linux-x86_64/settings" \
        "dist/linux-x86_64/settings"
    chmod +x "dist/linux-x86_64/{{exe_name}}" "dist/linux-x86_64/settings"
    @echo "Linux build: dist/linux-x86_64"

[linux]
install: linux-build
    mkdir -p "$HOME/.local/bin"
    cp "dist/linux-x86_64/{{exe_name}}" "$HOME/.local/bin/{{exe_name}}"
    cp "dist/linux-x86_64/settings" "$HOME/.local/bin/settings"
    chmod +x "$HOME/.local/bin/{{exe_name}}" "$HOME/.local/bin/settings"

# ---------------------------------------------------------------------------
# Utilities
# ---------------------------------------------------------------------------

clean:
    rm -rf dist

# ---------------------------------------------------------------------------
# Internal
# ---------------------------------------------------------------------------

_require-settings:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ ! -f "{{settings_dir}}/Justfile" ]; then
        echo "Error: Settings Justfile not found at {{settings_dir}}." >&2
        echo "Set SETTINGS_DIR, or clone rinodrops/settings to repos/settings." >&2
        exit 1
    fi
    if [ ! -f "{{settings_schema}}" ]; then
        echo "Error: schema not found at {{settings_schema}}" >&2
        exit 1
    fi

_settings-host:
    SETTINGS_SCHEMA="{{settings_schema}}" \
        just --justfile "{{settings_dir}}/Justfile" binary

_settings-arm64:
    SETTINGS_SCHEMA="{{settings_schema}}" \
        just --justfile "{{settings_dir}}/Justfile" binary-arm64

_settings-x86_64:
    SETTINGS_SCHEMA="{{settings_schema}}" \
        just --justfile "{{settings_dir}}/Justfile" binary-x86_64

_settings-win:
    SETTINGS_SCHEMA="{{settings_schema}}" \
        just --justfile "{{settings_dir}}/Justfile" settings-win-build

_settings-linux:
    SETTINGS_SCHEMA="{{settings_schema}}" \
        just --justfile "{{settings_dir}}/Justfile" settings-linux-build

_copy-host-settings dest:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p "{{dest}}"
    if [ -f "{{settings_dir}}/target/release/settings.exe" ]; then
        cp "{{settings_dir}}/target/release/settings.exe" "{{dest}}/settings.exe"
    elif [ -f "{{settings_dir}}/target/release/settings" ]; then
        cp "{{settings_dir}}/target/release/settings" "{{dest}}/settings"
    else
        echo "Error: Settings binary not found under {{settings_dir}}/target/release" >&2
        exit 1
    fi

_darwin-bundle arch rust_target:
    MACOSX_DEPLOYMENT_TARGET={{min_macos}} cargo build --release --target {{rust_target}}
    mkdir -p "dist/{{arch}}/{{app_name}}.app/Contents/MacOS"
    mkdir -p "dist/{{arch}}/{{app_name}}.app/Contents/Resources"
    cp "target/{{rust_target}}/release/{{exe_name}}" \
        "dist/{{arch}}/{{app_name}}.app/Contents/MacOS/{{exe_name}}"
    cp "{{settings_dir}}/target/{{rust_target}}/release/settings" \
        "dist/{{arch}}/{{app_name}}.app/Contents/MacOS/settings"
    sed "s/@VERSION@/{{version}}/g" assets/darwin/Info.plist \
        > "dist/{{arch}}/{{app_name}}.app/Contents/Info.plist"
