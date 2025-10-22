#[cfg(feature = "logging")]
use crate::emit::StringEmitter;
use crate::error::RepairError;
use crate::options::{EngineKind, Options};
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairLogEntry {
    pub position: usize,
    pub message: &'static str,
    pub context: String,
    pub path: Option<String>,
}

// Route to the selected engine at runtime (default: recursive-descent).
// When the `llm-compat` feature is not compiled, always fall back to recursive-descent.
#[inline]
fn engine_repair_to_string(input: &str, opts: &Options) -> Result<String, RepairError> {
    match opts.engine {
        EngineKind::Recursive => crate::parser::repair_to_string_impl(input, opts),
        EngineKind::LlmCompat => {
            #[cfg(feature = "llm-compat")]
            {
                return crate::engines::llm::repair_to_string_impl(input, opts);
            }
            #[cfg(not(feature = "llm-compat"))]
            {
                return crate::parser::repair_to_string_impl(input, opts);
            }
        }
        EngineKind::Auto => crate::parser::repair_to_string_impl(input, opts),
    }
}

#[inline]
fn engine_repair_to_writer<W: Write>(
    input: &str,
    opts: &Options,
    writer: &mut W,
) -> Result<(), RepairError> {
    match opts.engine {
        EngineKind::Recursive => crate::parser::repair_to_writer_impl(input, opts, writer),
        EngineKind::LlmCompat => {
            #[cfg(feature = "llm-compat")]
            {
                return crate::engines::llm::repair_to_writer_impl(input, opts, writer);
            }
            #[cfg(not(feature = "llm-compat"))]
            {
                return crate::parser::repair_to_writer_impl(input, opts, writer);
            }
        }
        EngineKind::Auto => crate::parser::repair_to_writer_impl(input, opts, writer),
    }
}

pub(crate) fn repair_to_string(input: &str, opts: &Options) -> Result<String, RepairError> {
    engine_repair_to_string(input, opts)
}

pub(crate) fn repair_to_writer_streaming<W: Write>(
    input: &str,
    opts: &Options,
    writer: &mut W,
) -> Result<(), RepairError> {
    engine_repair_to_writer(input, opts, writer)
}

#[cfg(feature = "logging")]
pub(crate) fn repair_to_string_with_log(
    input: &str,
    opts: &Options,
) -> Result<(String, Vec<RepairLogEntry>), RepairError> {
    // Force-enable logging for this call and return captured log entries
    let mut out = String::new();
    let mut emitter = StringEmitter::new(&mut out);
    let mut s = crate::parser::pre_trim_wrappers(input, opts);
    let mut logger = crate::parser::Logger::new(true, opts.log_json_path);
    crate::parser::parse_root_many(&mut s, opts, &mut emitter, &mut logger)?;
    Ok((out, logger.into_entries()))
}

#[cfg(not(feature = "logging"))]
pub(crate) fn repair_to_string_with_log(
    input: &str,
    opts: &Options,
) -> Result<(String, Vec<RepairLogEntry>), RepairError> {
    // Logging disabled at compile time: return repaired string with empty log
    let s = crate::parser::repair_to_string_impl(input, opts)?;
    Ok((s, Vec::new()))
}
