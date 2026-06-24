# claude-wordcloud

A time-travelling word cloud of your Claude Code sessions. A small Rust binary
reads your local session logs and builds a weekly timeline of word frequencies;
a single static HTML page animates it.

## How it works

```
~/.claude/projects/**/*.jsonl   →   claude-wordcloud   →   http://127.0.0.1:8080
```

The viewer (`web/index.html`) is embedded into the binary at build time and the
timeline is held in memory, so the compiled binary is the whole app — nothing
else needs to ship with it.

The extractor splits text into three **channels** — your *prompts*, the
assistant's *replies*, and its *thinking* — and scores each weekly bucket two
ways:

- **raw** — plain word counts (with stopwords removed)
- **tf-idf** — words *distinctive* to that week. This is what makes the
  time-travel interesting: each frame surfaces what was new, instead of the
  same head terms (`the`, `file`, `code`) every week.

The viewer toggles channel × metric live and plays through the weeks.

## Usage

```sh
cargo run --release                          # scan ~/.claude/projects, serve at :8080
cargo run --release -- /path/to/logs         # serve a custom logs dir
cargo run --release -- /path/to/logs out.js  # write buckets.js instead of serving
```

The serve modes open your browser automatically. The file-emit form is for
deploying the viewer statically (e.g. GitHub Pages): write `web/buckets.js`
alongside `web/index.html` and host the `web/` dir.

## Notes

- Serving binds `127.0.0.1:8080` and holds everything in memory — no files are
  written, and your session data never leaves the binary.
- The generated `web/buckets.js` (file-emit mode) is **gitignored** — it's
  derived from your private session history (which can include client/company
  terms). Don't commit it.
- Buckets are weekly (Monday-labelled). Tune `TOP_N` / bucket granularity /
  the stopword list in `src/main.rs`.
