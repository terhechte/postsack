# Postsack

## Provides a high level visual overview of swaths of email

### Performance

It currently parses 632383 emails in ~160 seconds, so roughly `4.000` emails per second. This excludes (for now) attachments.
Update: It currently parses 632115 emails in ~56 seconds, so roughly `11.000` emails per second. This excludes (for now) attachments.

## Open Issues

- [ ] rename repository to postsack
- [ ] add support for message filters (read etc), the UI is already there, the filters are not applied yet
- [ ] add a limited amount of tests..
- [ ] generate deletion rules based on stack?
- [ ] mbox is multiple files, e.g. inbox.mbox, sent.mbox. support this
- [x] support mbox
- [x] support apple mail
- [x] try `iced` or `druid` as well
- [ ] try re-opening a database...
- [ ] save config into sqlite
- [ ] store last opened sqlite file
- [ ] check if we get any values for the to_sender to_domain fields etc

## Future Options

- [ ] Add additional UI based on Druid, Iced or Native Cocoa
- [ ] maybe add blocking versions of the calls, too (in model)
- [ ] abstract over `Fields` and backend to have a generic way to display groupable information
- [ ] apply the window changes (no status etc) on startup, not just when loading main
- [ ] split up into multiple crates
- [ ] action when clicking an email?
- [ ] allow diving into splits/segments until there're no gropu bys anymore, but the last split can be opened full (to see the mails)
- [ ] remove unneeded dependencies and features

## Development

Generate a macOS bundle via [Cargo Bundle](https://github.com/burtonageo/cargo-bundle):

``` sh
cargo bundle --bin gui
```
