use super::*;

#[test]
fn st_jsonp_multiline_with_spaces_and_newlines() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["cb ", "(\n", " { \"a\" : 1 } ", " )\n"];
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    let tail = r.flush().unwrap();
    if !tail.is_empty() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
    let v: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn st_jsonp_name_with_underscore_space_before_paren() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["cb_1 ", "( ", "{a:1}", " )\n"]; // tolerate space before '('
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    let tail = r.flush().unwrap();
    if !tail.is_empty() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
    let v: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn st_nested_jsonp_wrappers() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["cb1(", "cb2(", "{a:1}", ")", ")\n"]; // should emit one object
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    let tail = r.flush().unwrap();
    if !tail.is_empty() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
    let v: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn st_jsonp_without_semicolon() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["cb(", "{b:2}", ")\n"]; // no trailing semicolon
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    let tail = r.flush().unwrap();
    if !tail.is_empty() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
    let v: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    assert_eq!(v, serde_json::json!({"b":2}));
}

#[test]
fn st_jsonp_name_with_digits_and_underscore() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["cb2_1(", "{x:3}", ")\n"]; // valid name
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    let tail = r.flush().unwrap();
    if !tail.is_empty() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
    let v: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    assert_eq!(v, serde_json::json!({"x":3}));
}

#[test]
fn st_jsonp_spaces_split() {
    let mut r = crate::StreamRepairer::new(Options::default());
    // tolerate spaces around '('
    let parts = ["cb ", "( ", "{a:1}", " )\n"];
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    let tail = r.flush().unwrap();
    if !tail.is_empty() {
        outs.push(tail);
    }
    assert_eq!(outs.len(), 1);
    let v: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn st_fenced_with_language_split() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["```", "json\n", "{a:1}", "\n```\n"]; // one object
    let s1 = r.push(parts[0]).unwrap();
    assert_eq!(s1, "");
    let s2 = r.push(parts[1]).unwrap();
    assert_eq!(s2, "");
    let s3 = r.push(parts[2]).unwrap();
    assert!(!s3.is_empty());
    let s4 = r.push(parts[3]).unwrap();
    assert_eq!(s4, "");
}

#[test]
fn st_two_fenced_blocks_sequential() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = [
        "```json\n",
        "{a:1}",
        "\n```\n",
        "```json\n",
        "{b:2}",
        "\n```\n",
    ];
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    assert_eq!(outs.len(), 2);
    let v1: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&outs[1]).unwrap();
    assert_eq!(v1, serde_json::json!({"a":1}));
    assert_eq!(v2, serde_json::json!({"b":2}));
}

#[test]
fn st_fenced_unknown_language_is_ignored() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["```javascript\n", "{a:1}", "\n```\n"]; // one object
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    assert_eq!(outs.len(), 1);
    let v: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}

#[test]
fn st_fence_plain_no_language_is_ignored() {
    let mut r = crate::StreamRepairer::new(Options::default());
    let parts = ["```\n", "{a:1}", "\n```\n"]; // one object
    let mut outs = Vec::new();
    for p in parts.iter() {
        let s = r.push(p).unwrap();
        if !s.is_empty() {
            outs.push(s);
        }
    }
    assert_eq!(outs.len(), 1);
    let v: serde_json::Value = serde_json::from_str(&outs[0]).unwrap();
    assert_eq!(v, serde_json::json!({"a":1}));
}
