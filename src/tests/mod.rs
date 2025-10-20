use super::*;

// Shared test helpers
fn lcg_sizes(seed: u64, len: usize) -> Vec<usize> {
    let mut x = seed;
    let mut out = Vec::new();
    let mut total = 0usize;
    while total < len {
        // LCG: constants from Numerical Recipes
        x = x.wrapping_mul(1664525).wrapping_add(1013904223);
        // chunk size in [1..16]
        let mut n = (((x >> 24) as usize) % 16) + 1;
        if total + n > len {
            n = len - total;
        }
        out.push(n);
        total += n;
    }
    out
}

fn chunk_by_char(s: &str, sizes: &[usize]) -> Vec<String> {
    let mut res = Vec::new();
    let mut iter = s.chars();
    for &n in sizes {
        if res
            .iter()
            .map(|p: &String| p.chars().count())
            .sum::<usize>()
            >= s.chars().count()
        {
            break;
        }
        let mut chunk = String::new();
        for _ in 0..n {
            if let Some(c) = iter.next() {
                chunk.push(c);
            } else {
                break;
            }
        }
        if !chunk.is_empty() {
            res.push(chunk);
        }
    }
    // Append remainder if any
    let rest: String = iter.collect();
    if !rest.is_empty() {
        res.push(rest);
    }
    res
}

// Submodules (topic-based)
mod arrays_objects_more;
mod comments_edge;
mod comments_ws;
mod core_non_streaming;
mod deep_malformed;
mod file_operations;
mod jsonp_fence;
mod logging_more;
mod logging_path;
mod ndjson;
mod non_streaming_misc;
mod numbers;
mod numbers_more;
mod objects_arrays;
mod python_compat;
mod python_parity;
mod python_parity_deep;
mod python_parity_fuzz;
mod python_parity_more;
mod stream_fuzz;
mod stream_fuzz_large;
mod streaming;
mod strings_escapes_more;
mod strings_regex_concat;
mod writer_streaming_more;
