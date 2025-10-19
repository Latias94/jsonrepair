use super::*;

#[test]
fn logging_path_key_with_space() {
    let opts = Options {
        logging: true,
        log_json_path: true,
        ..Default::default()
    };
    let input = "{'a b': undefined}";
    let (_out, log) = crate::repair_to_string_with_log(input, &opts).unwrap();
    let mut ok = false;
    for e in log {
        if e.message == "replaced undefined with null" {
            ok = e.path.as_deref() == Some("$[\"a b\"]");
        }
    }
    assert!(ok);
}
