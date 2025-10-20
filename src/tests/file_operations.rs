use crate::{Options, repair_to_string};
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_repair_from_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let broken_json = r#"{name: "John", age: 30}"#;
    temp_file.write_all(broken_json.as_bytes()).unwrap();
    
    // 测试文件读取和修复
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let fixed = repair_to_string(&content, &Options::default()).unwrap();
    
    let v: serde_json::Value = serde_json::from_str(&fixed).unwrap();
    assert_eq!(v["name"], "John");
    assert_eq!(v["age"], 30);
}

#[test]
fn test_repair_from_file_with_comments() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let broken_json = r#"
    {
        // User information
        name: "Alice",
        age: 25,
        /* Contact details */
        email: 'alice@example.com'
    }
    "#;
    temp_file.write_all(broken_json.as_bytes()).unwrap();
    
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let fixed = repair_to_string(&content, &Options::default()).unwrap();
    
    let v: serde_json::Value = serde_json::from_str(&fixed).unwrap();
    assert_eq!(v["name"], "Alice");
    assert_eq!(v["age"], 25);
    assert_eq!(v["email"], "alice@example.com");
}

#[test]
fn test_repair_from_file_incomplete() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let broken_json = r#"{"name": "Bob", "items": [1, 2, 3"#;
    temp_file.write_all(broken_json.as_bytes()).unwrap();
    
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let fixed = repair_to_string(&content, &Options::default()).unwrap();
    
    let v: serde_json::Value = serde_json::from_str(&fixed).unwrap();
    assert_eq!(v["name"], "Bob");
    assert!(v["items"].is_array());
    assert_eq!(v["items"].as_array().unwrap().len(), 3);
}

#[test]
fn test_repair_from_file_with_fence() {
    let mut temp_file = NamedTempFile::new().unwrap();
    // 使用单行的 fence 格式,避免被当作 NDJSON
    let broken_json = "```json\n{status: \"ok\", count: 42}\n```";
    temp_file.write_all(broken_json.as_bytes()).unwrap();

    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let fixed = repair_to_string(&content, &Options::default()).unwrap();

    let v: serde_json::Value = serde_json::from_str(&fixed).unwrap();
    assert_eq!(v["status"], "ok");
    assert_eq!(v["count"], 42);
}

#[test]
fn test_repair_from_file_unicode() {
    let mut temp_file = NamedTempFile::new().unwrap();
    let broken_json = r#"{name: "张三", city: '北京', message: "你好世界"}"#;
    temp_file.write_all(broken_json.as_bytes()).unwrap();

    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let opts = Options {
        ensure_ascii: false,
        ..Default::default()
    };
    let fixed = repair_to_string(&content, &opts).unwrap();

    let v: serde_json::Value = serde_json::from_str(&fixed).unwrap();
    assert_eq!(v["name"], "张三");
    assert_eq!(v["city"], "北京");
    assert_eq!(v["message"], "你好世界");
}

#[test]
fn test_repair_from_file_large() {
    let mut temp_file = NamedTempFile::new().unwrap();
    
    // 生成一个较大的 JSON 文件
    let mut broken_json = String::from("{users: [");
    for i in 0..100 {
        if i > 0 {
            broken_json.push_str(", ");
        }
        broken_json.push_str(&format!(
            "{{id: {}, name: 'User{}', active: true}}",
            i, i
        ));
    }
    broken_json.push_str("]}");
    
    temp_file.write_all(broken_json.as_bytes()).unwrap();
    
    let content = std::fs::read_to_string(temp_file.path()).unwrap();
    let fixed = repair_to_string(&content, &Options::default()).unwrap();
    
    let v: serde_json::Value = serde_json::from_str(&fixed).unwrap();
    assert!(v["users"].is_array());
    assert_eq!(v["users"].as_array().unwrap().len(), 100);
}

