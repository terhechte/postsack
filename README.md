<p align="center">
<img src="resources/github_logo.png" width="241" height="287" />
</p>

[![license](https://shields.io/badge/license-MIT-green)](https://github.com/terhechte/postsack/blob/main/LICENSE.md)
![Rust](https://github.com/<OWNER>/<REPOSITORY>/actions/workflows/rust.yml/badge.svg)


# Postsack

## Provides a high level visual overview of swaths of email

### Performance

It currently parses 632383 emails in ~160 seconds, so roughly `4.000` emails per second. This excludes (for now) attachments.
Update: It currently parses 632115 emails in ~56 seconds, so roughly `11.000` emails per second. This excludes (for now) attachments. (on M1)

## Open Issues

- [ ] Add some more code documentation to the different files / crates
- [ ] Logo
- [ ] Screenshots
- [ ] ci build buttons
- [ ] Web demo
- [ ] Demo Video
- [ ] run clippy again
- [ ] Documentation
  - [ ] briefly mention the email parser fork
  - [ ] explain how to use by exporting mails to mbox or google downloader
  - [ ] http://gmvault.org
  - [ ] web version / wasm
  - [ ] cargo bundle
  - [ ] native version / dependencies
  - [ ] Speed
  - [ ] (show the importer for 650k mails?)
  - [ ] add brief website to terhech.de

## Development

Generate a macOS bundle (requires [Cargo Bundle](https://github.com/burtonageo/cargo-bundle))

``` sh
./build_mac.sh
```

### Linux Dependencies

In order to build (and or run) on Linux, the following dependencies are needed:

#### Fedora

``` sh
# Development
sudo dnf install @development-tools glib cairo-devel pango-devel gdk-pixbux2-devel atk-devel gtk3 gtk3-devel libsqlite3x-devel
```

#### Ubuntu

``` sh
# Development
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev libsqlite3-dev
```

#### Windows

Windows supor is a bit shaky.

- [cargo bundle](https://github.com/burtonageo/cargo-bundle/issues/77) doesn't currently work on Windows
- [cargo wix should work](https://github.com/volks73/cargo-wix), but I could not get it to work
- `cargo build --release` works, but then the binary has no icon.
