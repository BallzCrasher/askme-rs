# askme-rs
CLI Client for asking questions in a reddit like fashion. made in Rust.

## Motivation
AskMe is a project that is supposed to handle questions from any user using a local system. 
Any user can signup and ask questions on AskMe. And any user can Answer any question after they signed up.
askme-rs is a twist on AskMe that is:
1) Written in Rust.
2) uses SQLite instead of normal file handling.
3) uses a reddit like fashion instead of a linear system

## Features
- Safe, Secure, and fast, Using SQLite and diesel-rs.
- Asking questions (posting threads) and answering (replying to) them.
- Answers can be anonymous of not (inherited from AskMe).
- A question (thread) can have multiple answers (replies).
- An answer (A reply) can have multiple answers (replies) to it.
- Accounts, threads and replies replies can be deleted

This is what makes askme-rs more powerful.
AskMe has only one answer to one question. Where askme-rs can have mutliple answers (replies) to one question (thread).
even replies can have more branching replies which adds to the reddit spirit of this project.

## Building
If you haven't already, install rust and Cargo :
- https://www.rust-lang.org/tools/install

then do:
```bash
git clone https://github.com/BallzCrasher/askme-rs.git
cd askme-rs
cargo build --release
cargo run 
```
## Notes
This is currently under slow development as it is a student project. But perhaps it could be a backend api for a rocket-rs server ?
there are few todos that needs polishing first:
1) make queries by index number. Not by id.
2) make it connect to an actual server instead of an SQLite database file.
3) ...
