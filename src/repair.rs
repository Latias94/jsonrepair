use crate::emit::StringEmitter;
use crate::error::RepairError;
use crate::options::Options;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairLogEntry {
    pub position: usize,
    pub message: &'static str,
    pub context: String,
    pub path: Option<String>,
}

pub(crate) fn repair_to_string(input: &str, opts: &Options) -> Result<String, RepairError> {
    crate::parser::repair_to_string_impl(input, opts)
}

pub(crate) fn repair_to_writer_streaming<W: Write>(
    input: &str,
    opts: &Options,
    writer: &mut W,
) -> Result<(), RepairError> {
    crate::parser::repair_to_writer_impl(input, opts, writer)
}

pub(crate) fn repair_to_string_with_log(
    input: &str,
    opts: &Options,
) -> Result<(String, Vec<RepairLogEntry>), RepairError> {
    // Force-enable logging for this call and return captured log entries
    // reuse parser pre-trim + emit with logger enabled
    let mut out = String::new();
    let mut emitter = StringEmitter::new(&mut out);
    let mut s = crate::parser::pre_trim_wrappers(input, opts);
    let mut logger = crate::parser::Logger::new(true, opts.log_json_path);
    crate::parser::parse_root_many(&mut s, opts, &mut emitter, &mut logger)?;
    Ok((out, logger.into_entries()))
}
