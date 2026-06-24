# claudegeist

**The zeitgeist of your Claude Code sessions.** A time-travelling word cloud of
everything you've been thinking out loud to a language model at 2am.

You talk to Claude more than you talk to most people. Every prompt is a tiny
confession of what you were stuck on, obsessed with, or quietly building.
`claudegeist` reads all of it back, buckets it by time, and plays it forward so
you can watch your own preoccupations rise, crest, and fade.

It's a word cloud. It's also a mirror. Mostly it's a word cloud.

```
        last week              this week             next sprint, probably
      ┌───────────┐          ┌───────────┐          ┌───────────┐
      │  migrate  │   ──▶    │  segfault │   ──▶    │  rewrite  │
      │  deadline │          │   WHY     │          │  in rust  │
      └───────────┘          └───────────┘          └───────────┘
```

---

## Install

macOS, one line, no `cargo`, no `brew`, no excuses:

```sh
curl -fsSL https://raw.githubusercontent.com/sebasv/claudegeist/main/install.sh | bash
```

Grabs a universal (Intel + Apple Silicon) binary from the latest release and
drops it in `/usr/local/bin`. Then:

```sh
claudegeist
```

It scans `~/.claude/projects`, builds your timeline, and opens the cloud at
`http://127.0.0.1:8080`. That's the whole onboarding.

> Building from source? `cargo run --release --` wherever you'd type `claudegeist`.

---

## How to read your own brain

Three knobs, top of the screen. Twist them while it plays.

| knob | options | what you're really looking at |
|------|---------|-------------------------------|
| **source** | `prompts` · `assistant` · `thinking` | your words / Claude's replies / Claude's *inner monologue* |
| **metric** | `distinctive` · `frequency` | what made each moment *different* / what you said most |
| **bucket** | `daily` · `weekly` · `sprint` | how zoomed-in time is |

Two of these deserve a footnote:

- **`distinctive` (tf-idf) is the good one.** Raw frequency just shows `the`,
  `file`, `function` forever — the vocabulary of work, identical every week.
  *Distinctive* surfaces what was new: the library you discovered Tuesday, the
  bug that ate Thursday. Play it on `distinctive` and you get a story. Play it on
  `frequency` and you get a screensaver.
- **`thinking` is the uncanny one.** It's the model's own reasoning trace. Watch
  it long enough and you start to wonder which of you is the assistant.

Hit **play**. Words don't snap between weeks — they swell and shrink
*continuously*, in place, so it actually feels like time passing instead of a
slideshow. Drag the scrubber to jump. Pick your speed.

---

## How it works

```
~/.claude/projects/**/*.jsonl  →  claudegeist  →  http://127.0.0.1:8080
        (your logs)              (one binary)       (the whole app)
```

1. **Read.** Every session is a JSONL log on your disk. `claudegeist` walks them
   all, pulls the text out of each message, and sorts it into three channels
   (your prompts, Claude's replies, Claude's thinking).
2. **Count.** Tokenise, drop stopwords and code-noise, tally per day.
3. **Roll up.** The daily counts collapse into weekly and sprint buckets, and
   each bucket gets scored two ways (raw count + tf-idf distinctiveness).
4. **Serve.** The viewer (`web/index.html`) is *baked into the binary* and the
   data lives in memory — so the compiled `claudegeist` is the entire product.
   Nothing to deploy, nothing to host, no `node_modules` in sight.

Rust + a stdlib HTTP server for the backend; d3 + d3-cloud for the cloud, with a
stable layout (words keep their spot) and per-frame size interpolation (so they
glide rather than jump).

### Usage cheatsheet

```sh
claudegeist                       # scan ~/.claude/projects, serve at :8080
claudegeist /path/to/logs         # point it at a different logs dir
claudegeist -b daily              # open on daily buckets (default: weekly; also: sprint)
claudegeist /path/to/logs out.js  # write the data to a file instead of serving
```

---

## Your secrets stay yours

It runs entirely on your machine. It binds to `127.0.0.1`, holds everything in
memory, and writes nothing by default — your half-formed 3am questions never
leave the binary. The repo is public, but it ships **code, not data**: the
generated `buckets.js` is gitignored on purpose. Your therapist's job is safe.

---

## A small existential note

You can't see your own attention. It's spent moment to moment and then it's
gone, with no record of where it went. This is one of the few records you have:
not a to-do list of what you *meant* to care about, but a trace of what actually
occupied you, week by week, in your own words.

Run it once a quarter. It's a surprisingly honest performance review.

---

<sub>Releases are cut by pushing a `v*` tag — CI builds the universal binary and
publishes it, which is what the installer fetches. Tune `TOP_N` and the stopword
list in `src/main.rs`.</sub>
