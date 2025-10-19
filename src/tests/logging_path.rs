use super::*;

#[test]
fn ns_logging_path_on_object_keys() {
    let opts = Options {
        logging: true,
        log_json_path: true,
        ..Default::default()
    };
    let input = "{a: True, b: undefined}";
    let (_out, log) = crate::repair_to_string_with_log(input, &opts).unwrap();
    // Expect entries for python keyword and undefined with proper paths
    let mut path_a = None;
    let mut path_b = None;
    for e in log {
        if e.message == "normalized python keyword" {
            path_a = e.path.clone();
        }
        if e.message == "replaced undefined with null" {
            path_b = e.path.clone();
        }
    }
    assert_eq!(path_a.as_deref(), Some("$[\"a\"]"));
    assert_eq!(path_b.as_deref(), Some("$[\"b\"]"));
}

#[test]
fn ns_logging_path_on_array_indices() {
    let opts = Options {
        logging: true,
        log_json_path: true,
        ..Default::default()
    };
    let input = "[undefined, True]";
    let (_out, log) = crate::repair_to_string_with_log(input, &opts).unwrap();
    let mut p0 = None;
    let mut p1 = None;
    for e in log {
        if e.message == "replaced undefined with null" {
            p0 = e.path.clone();
        }
        if e.message == "normalized python keyword" {
            p1 = e.path.clone();
        }
    }
    assert_eq!(p0.as_deref(), Some("$[0]"));
    assert_eq!(p1.as_deref(), Some("$[1]"));
}

#[test]
fn ns_logging_context_window_no_effect_on_path() {
    let opts = Options {
        logging: true,
        log_json_path: true,
        log_context_window: 2,
        ..Default::default()
    };
    let input = "{a: True}";
    let (_out, log) = crate::repair_to_string_with_log(input, &opts).unwrap();
    // find the python keyword normalization and ensure path matches regardless of small context window
    let mut found = false;
    for e in log {
        if e.message == "normalized python keyword" {
            found = e.path.as_deref() == Some("$[\"a\"]");
        }
    }
    assert!(found);
}

#[test]
fn ns_logging_nested_paths_object_array_object() {
    let opts = Options {
        logging: true,
        log_json_path: true,
        ..Default::default()
    };
    // { a: [ { b: undefined, c: True }, 0, { d: None } ] }
    let input = "{a:[{b:undefined,c:True},0,{d:None}]}";
    let (_out, log) = crate::repair_to_string_with_log(input, &opts).unwrap();
    let mut saw_b = false;
    let mut saw_c = false;
    let mut saw_d = false;
    for e in log {
        if let Some(p) = e.path.as_deref() {
            if p == "$[\"a\"][0][\"b\"]" {
                saw_b = true;
            }
            if p == "$[\"a\"][0][\"c\"]" {
                saw_c = true;
            }
            if p == "$[\"a\"][2][\"d\"]" {
                saw_d = true;
            }
        }
    }
    assert!(saw_b && saw_c && saw_d);
}

#[test]
fn ns_logging_path_array_of_arrays() {
    let opts = Options {
        logging: true,
        log_json_path: true,
        ..Default::default()
    };
    let input = "{a:[[],[0,undefined]]}";
    let (_out, log) = crate::repair_to_string_with_log(input, &opts).unwrap();
    let mut saw = false;
    for e in log {
        if e.message == "replaced undefined with null" {
            // Depending on key quoting in path, accept numeric index or quoted string
            let p = e.path.as_deref();
            saw = p == Some("$[\"a\"][1][1]") || p == Some("$[\"a\"][\"1\"][\"1\"]");
        }
    }
    assert!(saw);
}
