#!/bin/sh

set -e

# Compile for older macOS Versions
# See: https://users.rust-lang.org/t/compile-rust-binary-for-older-versions-of-mac-osx/38695/2
export MACOSX_DEPLOYMENT_TARGET=10.10

rm -rf ../target/release/bundle/osx/Postsack.app

# Build for x86 and ARM
cargo build --release --target=aarch64-apple-darwin
cargo build --release --target=x86_64-apple-darwin

# Combine into a fat binary

lipo -create ../target/aarch64-apple-darwin/release/postsack ../target/x86_64-apple-darwin/release/postsack -output postsack

# Perform Cargo bundle to create a macOS Bundle

cargo bundle --release

# Override bundle binary with the fat one
# Also: We want to have `Postsack` capitalized on macOS, so we rename

rm ../target/release/bundle/osx/Postsack.app/Contents/MacOS/postsack

mv ./postsack ../target/release/bundle/osx/Postsack.app/Contents/MacOS/Postsack

# Tell the Info.plist or binary is capitalized

/usr/libexec/PlistBuddy -c "Set :CFBundleExecutable Postsack" "../target/release/bundle/osx/Postsack.app/Contents/Info.plist"

# Create a zip file
cd ../target/release/bundle/osx/
/usr/bin/zip -5 -r ../../../postsack.zip ./Postsack.app
echo "Wrote zip file ../target/postsack.zip"
