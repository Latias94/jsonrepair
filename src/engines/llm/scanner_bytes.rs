// 简单的字节级 ASCII 连续段扫描器，用于加速未引号 key/值/字符串体的批量复制

#[inline]
pub fn ascii_run_key(bytes: &[u8]) -> usize {
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b >= 0x80 {
            break;
        }
        match b {
            b':' | b'}' | b',' | b'"' | b'\\' | b' ' | b'\t' | b'\n' | b'\r' => break,
            _ => i += 1,
        }
    }
    i
}

#[inline]
pub fn ascii_run_value(bytes: &[u8]) -> usize {
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b >= 0x80 {
            break;
        }
        match b {
            b',' | b'}' | b']' | b':' | b'"' | b'\\' | b' ' | b'\t' | b'\n' | b'\r' => break,
            _ => i += 1,
        }
    }
    i
}

#[inline]
pub fn ascii_run_string(bytes: &[u8], quote: u8) -> usize {
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b >= 0x80 {
            break;
        }
        if b == quote || b == b'"' || b == b'\\' {
            break;
        }
        i += 1;
    }
    i
}
