use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepairErrorKind {
    UnexpectedEnd,
    UnexpectedChar(char),
    ObjectKeyExpected,
    ColonExpected,
    InvalidUnicodeEscape,
    Parse(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairError {
    pub kind: RepairErrorKind,
    pub position: usize,
}

impl RepairError {
    pub fn new(kind: RepairErrorKind, position: usize) -> Self {
        Self { kind, position }
    }

    pub fn from_serde(what: &str, err: serde_json::Error) -> Self {
        let pos = err.line(); // coarse fallback
        Self {
            kind: RepairErrorKind::Parse(format!("serde_json {} error: {}", what, err)),
            position: pos,
        }
    }
}

impl fmt::Display for RepairError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            RepairErrorKind::UnexpectedEnd => {
                write!(f, "Unexpected end at position {}", self.position)
            }
            RepairErrorKind::UnexpectedChar(c) => {
                write!(
                    f,
                    "Unexpected character {:?} at position {}",
                    c, self.position
                )
            }
            RepairErrorKind::ObjectKeyExpected => {
                write!(f, "Object key expected at position {}", self.position)
            }
            RepairErrorKind::ColonExpected => {
                write!(f, "Colon expected at position {}", self.position)
            }
            RepairErrorKind::InvalidUnicodeEscape => {
                write!(f, "Invalid unicode escape at position {}", self.position)
            }
            RepairErrorKind::Parse(msg) => write!(f, "{} at position {}", msg, self.position),
        }
    }
}

impl std::error::Error for RepairError {}
