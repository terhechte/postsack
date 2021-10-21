# Gmail DB

## Fast

It currently parses 632383 emails in ~160 seconds, so roughly `4.000` emails per second. This excludes (for now) attachments.
Update: It currently parses 632115 emails in ~56 seconds, so roughly `11.000` emails per second. This excludes (for now) attachments.

## Open Issues
- [ ] make the rectangles look nicer
- [ ] improve the egui UI
- [ ] generate deletion rules based on stack?
- [ ] try the segment range slider with two sides
- [ ] support mbox
- [ ] support apple mail
- [ ] try `iced` or `druid` as well
- [ ] maybe add blocking versions of the calls, too (in model)
- [ ] abstract over `Fields` and backend

- [ ] open database support
- [ ] add the read, send, filtres
- [ ] mbox is multiple files
- [ ] rename
- [ ] store last opened sqlite file
- [ ] save config into sqlite
- [ ] split up into multiple crates