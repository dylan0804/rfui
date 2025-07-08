# rfui

Terminal file finder with live preview.

## What it does

Search through your filesystem interactively. Type patterns, get results instantly. Preview files with syntax highlighting. Navigate with keyboard shortcuts. Works fast on large directory trees.

Built with Rust using nucleo for fuzzy matching, ignore for fast traversal, bat for previews, and ratatui for the interface.

## Installation

```bash
cargo install rfui
```

Or build from source:

```bash
git clone https://github.com/dylan0804/rfui
cd rfui
cargo install --path .
```

Requires Rust and bat for syntax highlighting.

## Usage

```bash
rfui
```

Type search patterns in the interface:
- `config` - find files containing "config"
- `config -k f` - only files, not directories  
- `test -d 2` - limit search depth to 2 levels
- `log -H` - include hidden files

```
-k, --kind <TYPE>        Filter by type (f/file, d/directory)
-d, --max-depth <NUM>    Maximum search depth
-H, --hidden             Include hidden files
-s, --case-sensitive     Case sensitive search  
-t, --threads <NUM>      Number of search threads
```

## Key bindings

```
Navigation:
  ↑/↓,                  Navigate results
  ←/→                   Move cursor in search
  Enter                 Execute search
  Esc                   Quit

Preview:
  Ctrl+K/J              Scroll preview vertically
  Ctrl+H/L              Scroll preview horizontally  
  Ctrl+U/D              Resize preview/results

Other:
  Ctrl+Y                Copy file path to clipboard
  /help                 Show help screen
```

## Implementation

Built with ratatui for the terminal interface, nucleo for fuzzy matching, ignore for fast directory traversal, and bat for syntax highlighting. Uses crossterm for cross-platform terminal handling.

## License

MIT
