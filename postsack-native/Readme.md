# Postsack Native

This crate embeds `ps-core`, `ps-database` and `ps-importer` to build a native version of Postsack for macOS, Linux and Windows.

A native version can be run via `cargo run` or `cargo build`, however a proper application requires installing [cargo bundle](https://github.com/burtonageo/cargo-bundle):

``` sh
cargo install cargo-bundle
```

## macOS

For macOS, there is a script that will build a fat binary (with aarch64 and x86_64) into a macOS `Postsack.app` bundle.
Just running `cargo bundle --release` will also create an app bundle, but it will only contain the host architecture.

``` sh
./build_mac.sh
```

## Linux

Depending on your Linux installation, you will need to install a couple of dependencies:

### Fedora Based

``` sh
sudo dnf install @development-tools glib cairo-devel pango-devel gdk-pixbux2-devel atk-devel gtk3 gtk3-devel libsqlite3x-devel
```

### Ubuntu based

``` sh
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev libsqlite3-dev
```

### Arch based

Thanks to Emilio Reggi, [there's an Arch package here](https://aur.archlinux.org/packages/postsack-bin/)

### NixOS

There's a `shell.nix` in this directory. It can be used to run Postsack on NixOS via:

``` sh
> nix-shell shell.nix
$ cargo run
```

### Building

``` sh
./build_linux.sh
```

## Windows

Building for Windows works but doesn't create a proper application (e.g. with an icon) and also doesn't create a proper installer.

- [cargo bundle](https://github.com/burtonageo/cargo-bundle/issues/77) doesn't currently work on Windows
- [cargo wix should work](https://github.com/volks73/cargo-wix), but I could not get it to work
- `cargo build --release` works, but then the binary has no icon.

Running `build_windows.bat` therefore will currently just run `cargo build --release`

``` cmd
build_windows.bat
```
