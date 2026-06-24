# claudegeist

The *zeitgeist* of your Claude Code sessions — a time-travelling word cloud of
what you've been working on. A small Rust binary reads your local session logs
and builds a timeline of word frequencies; a single embedded HTML page animates
it.

## How it works

```
~/.claude/projects/**/*.jsonl   →   claudegeist   →   http://127.0.0.1:8080
```

The viewer (`web/index.html`) is embedded into the binary at build time and the
timeline is held in memory, so the compiled binary is the whole app — nothing
else needs to ship with it.

The extractor splits text into three **channels** — your *prompts*, the
assistant's *replies*, and its *thinking* — and scores each time bucket two
ways:

- **raw** — plain word counts (with stopwords removed)
- **tf-idf** — words *distinctive* to that week. This is what makes the
  time-travel interesting: each frame surfaces what was new, instead of the
  same head terms (`the`, `file`, `code`) every week.

As you play through time, words resize and fade **in place** (stable layout, no
re-shuffling), so you can actually see which terms grew and faded.

The viewer toggles channel × metric live and plays through the timeline.

## Install (macOS, no cargo needed)

```sh
curl -fsSL https://raw.githubusercontent.com/sebasv/claudegeist/main/install.sh | bash
```

Downloads a universal (Intel + Apple Silicon) binary from the latest release,
drops it in `/usr/local/bin`, and you're done. Then just run `claudegeist`.

## Usage

```sh
claudegeist                       # scan ~/.claude/projects, serve at :8080
claudegeist /path/to/logs         # serve a custom logs dir
claudegeist -b daily              # bucket by day (default: weekly; also: sprint)
claudegeist /path/to/logs out.js  # write buckets.js instead of serving
```

(Building from source instead? Swap `claudegeist` for `cargo run --release --`.)

**Bucketing** (`-b` / `--bucket`): `daily`, `weekly` (default, Monday-labelled),
or `sprint` (two-week blocks). More buckets = more animation frames; fewer =
broader strokes.

The serve modes open your browser automatically. The file-emit form is for
deploying the viewer statically (e.g. GitHub Pages): write `web/buckets.js`
alongside `web/index.html` and host the `web/` dir.

## Notes

- Serving binds `127.0.0.1:8080` and holds everything in memory — no files are
  written, and your session data never leaves the binary.
- The generated `web/buckets.js` (file-emit mode) is **gitignored** — it's
  derived from your private session history (which can include client/company
  terms). Don't commit it.
- Tune `TOP_N` / the stopword list in `src/main.rs`.
- Releases are cut by pushing a `v*` tag — CI builds the universal binary and
  publishes it, which is what the install one-liner fetches.
