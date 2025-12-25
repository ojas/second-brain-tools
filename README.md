# Ojas's Second Brain Tools

Learning Rust. Building a CLI tool to help me manage my second brain.

```bash
# test
cargo test

# create an example tree.json file in the format we want.
tree -J --du -D --timefmt "%Y-%m-%d" . > tree_example.json

# run
cargo run tree_example.json

# build package binary
cargo build

# run (default binary target)
target/debug/second-brain-tools ~/sync/trees/tree_sync.json > ~/sync/trees/tree_sync.txt
```

Note: [crates.io](https://crates.io/) token stored locally in `.crates_io_token`. Take care.
