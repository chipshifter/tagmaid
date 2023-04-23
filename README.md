<h1>TagMaid, a tagging based file explorer</h1>
<div align="center">
    <img src="./logo.png" alt="TagMaid logo" height="10%" width="10%">
</div>
<br />

TagMaid is a file explorer tool that lets you sort your files using manually defined tags.
It aims to solve the problem of endlessly nested folders when trying to sort and search through all your files, 
which becomes quite complex and inefficient to do when your archive grows.

TagMaid is built with ease of use in mind. How it works is straightforward: to each file you add into TagMaid, you assign it tags. 
Then, using the search tool, search for one (or more) tags, and set of files that contains the tags that you are looking for will
appear before your eyes. 

One negative aspect of this solution is that you need to add most tags yourself and this can take some time at first.

### !! This software is in very early stage and not suitable for a stable public release !!

## 0.2 - The search update

(TEST)

## Features

TagMaid is written completely in Rust. The main dependencies are `egui` for the GUI and `rusqlite` for the SQLite database.
There are very few features as of yet, since the software is still in the "proof of concept" stage.

## Build

Building should be as simple as running `cargo build --release`. You can also simply run `cargo run` to test
the software.

**You may also need to install the `pkg-config` and `libgtk-3-dev` system packages to build some dependencies the first time.**

## Debugging

Here are cargo features used for debugging and development purposes:
- `cargo run --features ui_debug` adds a "Debug" tab to the UI for `egui`'s debug options
- `cargo run --features import_samples` automatically imports files in `tag-maid/src/samples` to the database
- `cargo run --features manual` for manual database intervention 

You can activate logging using the `RUST_LOG` environment variable:
- `info` level (e.g. running `RUST_LOG=info cargo run`) will give you most function calls
- `debug` level gives you more thorough logging
