# Postsack

## Provides a high level visual overview of swaths of email

### Performance

It currently parses 632383 emails in ~160 seconds, so roughly `4.000` emails per second. This excludes (for now) attachments.
Update: It currently parses 632115 emails in ~56 seconds, so roughly `11.000` emails per second. This excludes (for now) attachments. (on M1)

## Open Issues

- [ ] build static linux binary via docker: Via Github Actions?
- [ ] try to build a static windows binary: Via Github Actions?
- [ ] try to build a macos binary: Via Github Actions?
- [ ] Demo Video
- [ ] Documentation
- [ ] wasm build?


## Windows Issues

- [ ] No Outlook support
- [ ] The `apple importer` fails
- [ ] Very much untested (it does run though)

## Future Options

- [ ] Add additional UI based on Druid, Iced or Native Cocoa
- [ ] maybe add blocking versions of the calls, too (in model)
- [ ] abstract over `Fields` and backend to have a generic way to display groupable information
- [ ] apply the window changes (no status etc) on startup, not just when loading main
- [ ] split up into multiple crates
- [ ] action when clicking an email?
- [ ] support light theme
- [ ] allow diving into splits/segments until there're no gropu bys anymore, but the last split can be opened full (to see the mails)
- [ ] remove unneeded dependencies and features
- [ ] add support for generating mail deletion rules based on the visible mails
- [ ] support more mail formats:
  - [ ] outlook
  - [ ] notmuch
  - [ ] maildir

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


