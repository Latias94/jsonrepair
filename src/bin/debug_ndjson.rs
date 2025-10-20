// Temporary debug binary to inspect NDJSON aggregate output.
// Note: This file is for local debugging and can be removed later.
use jsonrepair::{Options, StreamRepairer};

fn main() {
    let mut corpus = String::new();
    for i in 0..30usize {
        corpus.push_str(&format!("{{a:{}}}\n", i));
    }
    let mut opts = Options::default();
    opts.stream_ndjson_aggregate = true;
    let mut r = StreamRepairer::new(opts);
    // feed in oddly sized chunks to simulate streaming
    let sizes = [1usize, 2, 3, 5, 8, 13, 21, 34];
    let mut idx = 0usize;
    while idx < corpus.len() {
        for &sz in &sizes {
            if idx >= corpus.len() { break; }
            let end = (idx + sz).min(corpus.len());
            let chunk = &corpus[idx..end];
            let s = r.push(chunk).expect("push ok");
            if !s.is_empty() {
                eprintln!("unexpected mid-output: {}", s);
            }
            idx = end;
        }
    }
    let ret = r.flush().expect("flush ok");
    println!("AGGRET:{}", ret);
    // Also test non-streaming on a single sample
    let single = "{a:0}";
    let fixed = jsonrepair::repair_to_string(single, &Options::default()).expect("repair ok");
    println!("SINGLE:{}", fixed);

    // streaming no-aggregate check on a single line
    let mut r2 = StreamRepairer::new(Options::default());
    let inp = "{a:0}\n";
    for ch in inp.chars() {
        let s = r2.push(&ch.to_string()).expect("push ok");
        if !s.is_empty() { println!("EMIT:{}", s); }
    }
    let tail2 = r2.flush().expect("flush ok");
    if !tail2.is_empty() { println!("TAIL:{}", tail2); }

    // Inspect embedded quotes case
    let s = "[\"lorem \"ipsum\" sic\"]";
    let out = jsonrepair::repair_to_string(s, &Options::default()).expect("ok");
    println!("EMBED:{}", out);

    // Inspect negative leading dot
    let out2 = jsonrepair::repair_to_string("{a:-.5}", &Options::default()).expect("ok");
    println!("NEG_LDOT:{}", out2);

    // Writer basic NDJSON
    let mut corpus = String::new();
    for i in 0..20usize { corpus.push_str(&format!("{{a:{}}}\n", i)); }
    let mut r3 = StreamRepairer::new(Options::default());
    let mut buf = Vec::new();
    for ch in corpus.chars() {
        let mut s = String::new(); s.push(ch);
        r3.push_to_writer(&s, &mut buf).expect("writer ok");
    }
    r3.flush_to_writer(&mut buf).expect("flush writer ok");
    let out3 = String::from_utf8(buf).unwrap();
    println!("WRITER:{}", out3);
}
