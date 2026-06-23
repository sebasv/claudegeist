# claude-wordcloud

A time-travelling word cloud of your Claude Code sessions. A small Rust binary
reads your local session logs and builds a weekly timeline of word frequencies;
a single static HTML page animates it.

## How it works

```
~/.claude/projects/**/*.jsonl   →   claude-wordcloud   →   web/buckets.js   →   web/index.html
```

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
cargo run --release            # reads ~/.claude/projects, writes web/buckets.js
cd web && python3 -m http.server   # then open http://localhost:8000
```

`web/buckets.js` defines `window.BUCKETS`, so you can also just open
`web/index.html` directly from disk — no server needed.

Point it elsewhere or change the output path with args:

```sh
cargo run --release -- /path/to/logs web/buckets.js
```

## Notes

- `web/buckets.js` is **gitignored** — it's derived from your private session
  history (which can include client/company terms). Regenerate it locally
  rather than committing it.
- Buckets are weekly (Monday-labelled). Tune `TOP_N` / bucket granularity /
  the stopword list in `src/main.rs`.
