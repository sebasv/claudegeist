//! Reads Claude Code session logs (~/.claude/projects/**/*.jsonl) and emits a
//! time-bucketed word frequency timeline as web/buckets.js for the viewer.
//!
//! Three channels (prompts / assistant text / thinking) x two metrics (raw
//! count / tf-idf) are precomputed; the frontend just toggles between them.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde_json::{json, Value};

const TOP_N: usize = 80;
const MIN_LEN: usize = 3;
const MAX_LEN: usize = 30;
const CHANNELS: [&str; 3] = ["prompts", "assistant", "thinking"];

fn main() {
    let mut args = std::env::args().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| dirs_home().join(".claude/projects"));
    let output = args
        .next()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("web/buckets.js"));

    let stops = stopwords();

    // channel -> bucket-label -> word -> count
    let mut counts: HashMap<&str, HashMap<String, HashMap<String, u64>>> =
        CHANNELS.iter().map(|c| (*c, HashMap::new())).collect();

    let mut files = Vec::new();
    collect_jsonl(&input, &mut files);
    eprintln!("scanning {} session files under {}", files.len(), input.display());

    let mut records = 0u64;
    for f in &files {
        let Ok(text) = fs::read_to_string(f) else { continue };
        for line in text.lines() {
            let Ok(v) = serde_json::from_str::<Value>(line) else { continue };
            let Some(ts) = v.get("timestamp").and_then(Value::as_str) else { continue };
            let Some(bucket) = week_label(ts) else { continue };
            let typ = v.get("type").and_then(Value::as_str).unwrap_or("");
            let content = v.pointer("/message/content");

            for (channel, text) in extract(typ, content) {
                if text.is_empty() {
                    continue;
                }
                records += 1;
                let bmap = counts.get_mut(channel).unwrap().entry(bucket.clone()).or_default();
                for tok in tokenize(&text, &stops) {
                    *bmap.entry(tok).or_insert(0) += 1;
                }
            }
        }
    }
    eprintln!("ingested {} text records", records);

    // union of all bucket labels, sorted ascending
    let mut labels: Vec<String> = counts
        .values()
        .flat_map(|m| m.keys().cloned())
        .collect();
    labels.sort();
    labels.dedup();

    let mut data = serde_json::Map::new();
    for channel in CHANNELS {
        let per_bucket = &counts[channel];
        // document frequency: how many buckets contain each word
        let mut df: HashMap<&str, usize> = HashMap::new();
        for words in per_bucket.values() {
            for w in words.keys() {
                *df.entry(w).or_insert(0) += 1;
            }
        }
        let n_docs = per_bucket.len().max(1) as f64;

        let frames: Vec<Value> = labels
            .iter()
            .map(|label| {
                let Some(words) = per_bucket.get(label) else {
                    return json!({"raw": [], "tfidf": []});
                };
                let total: u64 = words.values().sum();
                let total = total.max(1) as f64;

                let raw = top_n(words.iter().map(|(w, &c)| (w.clone(), c as f64)));
                let tfidf = top_n(words.iter().map(|(w, &c)| {
                    let tf = c as f64 / total;
                    let idf = (n_docs / df[w.as_str()] as f64).ln() + 1.0;
                    (w.clone(), round(tf * idf))
                }));
                json!({"raw": raw, "tfidf": tfidf})
            })
            .collect();
        data.insert(channel.to_string(), Value::Array(frames));
    }

    let out = json!({
        "bucket_unit": "week",
        "buckets": labels,
        "channels": CHANNELS,
        "data": data,
    });

    if let Some(parent) = output.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let js = format!("window.BUCKETS = {};\n", serde_json::to_string(&out).unwrap());
    fs::write(&output, js).expect("write output");
    eprintln!("wrote {} ({} weekly buckets)", output.display(), labels.len());
}

/// Pulls (channel, text) pairs out of one record's message content.
fn extract(typ: &str, content: Option<&Value>) -> Vec<(&'static str, String)> {
    match (typ, content) {
        ("user", Some(Value::String(s))) => vec![("prompts", s.clone())],
        ("user", Some(Value::Array(blocks))) => {
            // text blocks are real prompts; tool_result blocks are excluded
            vec![("prompts", join_blocks(blocks, "text"))]
        }
        ("assistant", Some(Value::Array(blocks))) => vec![
            ("assistant", join_blocks(blocks, "text")),
            ("thinking", join_blocks(blocks, "thinking")),
        ],
        _ => vec![],
    }
}

fn join_blocks(blocks: &[Value], kind: &str) -> String {
    blocks
        .iter()
        .filter(|b| b.get("type").and_then(Value::as_str) == Some(kind))
        .filter_map(|b| b.get(kind).and_then(Value::as_str))
        .collect::<Vec<_>>()
        .join(" ")
}

fn tokenize(text: &str, stops: &std::collections::HashSet<&str>) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= MIN_LEN && w.len() <= MAX_LEN)
        .map(str::to_lowercase)
        .filter(|w| !w.chars().all(|c| c.is_ascii_digit()))
        .filter(|w| !stops.contains(w.as_str()))
        .collect()
}

fn top_n(iter: impl Iterator<Item = (String, f64)>) -> Vec<Value> {
    let mut v: Vec<(String, f64)> = iter.collect();
    v.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then(a.0.cmp(&b.0)));
    v.truncate(TOP_N);
    v.into_iter().map(|(w, s)| json!([w, s])).collect()
}

fn round(x: f64) -> f64 {
    (x * 100000.0).round() / 100000.0
}

/// ISO-week bucket, labelled by the Monday of that week (YYYY-MM-DD).
fn week_label(ts: &str) -> Option<String> {
    let dt: DateTime<Utc> = ts.parse().ok()?;
    let iso = dt.iso_week();
    let monday = NaiveDate::from_isoywd_opt(iso.year(), iso.week(), chrono::Weekday::Mon)?;
    Some(monday.format("%Y-%m-%d").to_string())
}

fn collect_jsonl(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_jsonl(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("jsonl") {
            out.push(p);
        }
    }
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

fn stopwords() -> std::collections::HashSet<&'static str> {
    // English function words + Claude Code / coding boilerplate that otherwise
    // dominates every frame and drowns the signal.
    const WORDS: &str = "\
the and for are but not you all any can her was one our out day get has him his how man new now old see two way who boy did its let put say she too use that with have this will your from they know want been good much some time very when come here just like long make many over such take than them well were what
about would there their which could other into more then your also your may these said each she does been your most some what your only over also back after work first well way even new want because any these give day us
i'm i've you're we're they're it's that's don't didn't doesn't can't won't isn't aren't wasn't weren't shouldn't wouldn't couldn't let's
let me okay sure yeah yes no nope yep need needs needed should would could might must will shall does did doing done go going gone goes
the this that with from have will your they been were what when where which while would there their about into your over then than them
file files line lines code add added adding change changed changes update updated run running ran test tests function functions method use using used value values name names type types return returns set get got make made new like just also need want should would could now then here there this that these those
true false null none nil void int str string bool list dict map vec
http https www com org github linear app issue pull com www
def fn pub let var const async await impl mod use crate self mut ref dyn box";
    WORDS.split_whitespace().collect()
}
