# Ojas's Second Brain Tools

Learning Rust. Building a CLI tool to help me manage my second brain.

## Available Commands

- `tree` - Parse tree JSON and output TSV format (path, size, timestamp)
- `bookmarks` - Generate and sync Netscape-style bookmark HTML index files
- `pixie` - Process photo albums with resizing and metadata generation
- `vault` - Convert Obsidian vault to publishable markdown

## Quick Start

```bash
# Show all commands
target/debug/second-brain-tools --help

# Show help for specific command
target/debug/second-brain-tools vault --help
```

## Usage Examples

```bash
# Test
cargo test

# Build
cargo build

# Tree command - parse tree JSON output
tree -J --du -D --timefmt "%Y-%m-%d" . > tree_example.json
cargo run -- tree tree_example.json
target/debug/second-brain-tools tree ~/sync/trees/tree_sync.json > ~/sync/trees/tree_sync.txt

# Bookmarks command - generate bookmark index
target/debug/second-brain-tools bookmarks /path/to/folder
target/debug/second-brain-tools bookmarks /path/to/folder --recursive

# Pixie command - process photo albums
target/debug/second-brain-tools pixie --config pixie.yaml

# Vault command - convert Obsidian vault to publishable markdown
target/debug/second-brain-tools vault ~/Projects/my-vault -o ~/site/content
```

## Notes

- [crates.io](https://crates.io/) token stored locally in `.crates_io_token`. Take care.
- Vault command only processes files with `publish: true` in frontmatter
- Vault command converts `[[wikilinks]]` to standard markdown links
