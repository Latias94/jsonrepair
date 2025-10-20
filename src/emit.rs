use crate::error::{RepairError, RepairErrorKind};
use std::io::Write;

pub type JRResult<T> = Result<T, RepairError>;

pub trait Emitter {
    fn emit_str(&mut self, s: &str) -> JRResult<()>;
    fn emit_char(&mut self, c: char) -> JRResult<()> {
        let mut buf = [0u8; 4];
        let s = c.encode_utf8(&mut buf);
        self.emit_str(s)
    }
}

pub struct StringEmitter<'a> {
    out: &'a mut String,
}

impl<'a> StringEmitter<'a> {
    pub fn new(out: &'a mut String) -> Self {
        Self { out }
    }
}

impl<'a> Emitter for StringEmitter<'a> {
    fn emit_str(&mut self, s: &str) -> JRResult<()> {
        self.out.push_str(s);
        Ok(())
    }
}

pub struct WriterEmitter<'a, W: Write> {
    pub w: &'a mut W,
    buf: Vec<u8>,
}

impl<'a, W: Write> WriterEmitter<'a, W> {
    pub fn with_capacity(w: &'a mut W, cap: usize) -> Self {
        Self { w, buf: Vec::with_capacity(cap) }
    }
    pub fn flush_all(&mut self) -> JRResult<()> {
        if !self.buf.is_empty() {
            self.w
                .write_all(&self.buf)
                .map_err(|e| RepairError::new(RepairErrorKind::Parse(format!("io write error: {}", e)), 0))?;
            self.buf.clear();
        }
        Ok(())
    }
}

impl<'a, W: Write> Emitter for WriterEmitter<'a, W> {
    fn emit_str(&mut self, s: &str) -> JRResult<()> {
        self.buf.extend_from_slice(s.as_bytes());
        Ok(())
    }
}
