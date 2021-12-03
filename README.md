# Postsack

## Provides a high level visual overview of swaths of email

### Performance

It currently parses 632383 emails in ~160 seconds, so roughly `4.000` emails per second. This excludes (for now) attachments.
Update: It currently parses 632115 emails in ~56 seconds, so roughly `11.000` emails per second. This excludes (for now) attachments.

## Open Issues

- [ ] rename repository to postsack
- [x] add support for message filters (read etc), the UI is already there, the filters are not applied yet
  - [x] the filters for tags should allow selecting from the existing tags?
- [x] mbox is multiple files, e.g. inbox.mbox, sent.mbox. support this
- [x] support mbox
- [x] support apple mail
- [x] try re-opening a database...
- [x] save config into sqlite
- [x] store last opened sqlite file
- [ ] check if we get any values for the to_sender to_domain fields etc
- [x] update to egui 15
- [ ] add more tests
- [ ] build static linux binary via docker
- [ ] try to build a static windows binary

## Linux Issues

- [x] broken background color
- [x] can't switch import format
- [x] mbox support for selecting a folder of mbox files (inbox etc)
- [ ] weird sizing behaviour on startup
- [x] background color in startup screen is transparent
- [ ] current tests fail under linux

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

Generate a macOS bundle via [Cargo Bundle](https://github.com/burtonageo/cargo-bundle):

``` sh
cargo bundle --bin gui
```

### Linux Dependencies

In order to build (and or run) on Linux, the following dependencies are needed:

#### Fedora

``` sh
# Development
sudo dnf install @development-tools glib cairo-devel pango-devel gdk-pixbux2-devel atk-devel gtk3 gtk3-devel libsqlite3x-devel
```

#### Ubuntu (currently untested)


``` sh
# Development
sudo apt-get install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libspeechd-dev libxkbcommon-dev libssl-dev libsqlite3-dev
```


