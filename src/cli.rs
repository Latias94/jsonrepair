use crate::{
    LeadingZeroPolicy, Options, StreamRepairer, repair_to_string, repair_to_writer_streaming,
};
use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};

fn print_help(program: &str) {
    eprintln!(
        "Usage: {prog} [OPTIONS] [INPUT]\n\
         \n\
         INPUT: optional input file. When omitted, reads from stdin.\n\
         \n\
         Options:\n\
           -o, --output FILE         Write output to FILE (default stdout)\n\
               --in-place            Overwrite INPUT file (implies non-streaming)\n\
               --stream              Stream while parsing (lower memory).\n\
               --chunk-size BYTES    Chunk size for streaming (default 65536)\n\
               --ndjson-aggregate    Aggregate NDJSON values into a single array (streaming)\n\
               --pretty              Pretty-print output (non-streaming path)\n\
               --ensure-ascii        Escape non-ASCII as \\uXXXX\n\
               --no-python-keywords  Disable Python True/False/None normalization\n\
               --no-undefined-null   Disable undefined -> null repair\n\
               --no-fence            Disable fenced code block stripping\n\
               --no-hash-comments    Disable # line comment tolerance\n\
               --no-nonfinite-null   Disable NaN/Infinity -> null normalization\n\
               --leading-zero POLICY Keep|Quote (default Keep)\n\
           -h, --help                Show this help\n",
        prog = program
    );
}

fn parse_args() -> (Options, CliMode) {
    let mut args: Vec<String> = env::args().collect();
    let program = args
        .first()
        .cloned()
        .unwrap_or_else(|| "jsonrepair".to_string());
    args.remove(0);

    let mut opts = Options::default();
    let mut input: Option<String> = None;
    let mut output: Option<String> = None;
    let mut in_place = false;
    let mut stream = false;
    let mut chunk_size: usize = 65536;
    let mut pretty = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help(&program);
                std::process::exit(0);
            }
            "-o" | "--output" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing FILE for --output");
                    std::process::exit(2);
                }
                output = Some(args[i].clone());
            }
            "--in-place" => {
                in_place = true;
            }
            "--stream" => {
                stream = true;
            }
            "--chunk-size" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing BYTES for --chunk-size");
                    std::process::exit(2);
                }
                chunk_size = args[i].parse().unwrap_or(65536);
            }
            "--ndjson-aggregate" => {
                opts.stream_ndjson_aggregate = true;
            }
            "--pretty" => {
                pretty = true;
            }
            "--ensure-ascii" => {
                opts.ensure_ascii = true;
            }
            "--no-python-keywords" => {
                opts.allow_python_keywords = false;
            }
            "--no-undefined-null" => {
                opts.repair_undefined = false;
            }
            "--no-fence" => {
                opts.fenced_code_blocks = false;
            }
            "--no-hash-comments" => {
                opts.tolerate_hash_comments = false;
            }
            "--no-nonfinite-null" => {
                opts.normalize_js_nonfinite = false;
            }
            "--leading-zero" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing POLICY for --leading-zero");
                    std::process::exit(2);
                }
                match args[i].to_lowercase().as_str() {
                    "keep" => opts.leading_zero_policy = LeadingZeroPolicy::KeepAsNumber,
                    "quote" => opts.leading_zero_policy = LeadingZeroPolicy::QuoteAsString,
                    other => {
                        eprintln!("Unknown leading-zero policy: {}", other);
                        std::process::exit(2);
                    }
                }
            }
            "--compat" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing PROFILE for --compat");
                    std::process::exit(2);
                }
                match args[i].to_lowercase().as_str() {
                    "python" => {
                        opts.compat_python_friendly = true;
                        opts.number_tolerance_leading_dot = true;
                        opts.number_tolerance_trailing_dot = true;
                        opts.number_tolerance_incomplete_exponent = true;
                        opts.word_comment_markers =
                            vec!["COMMENT".to_string(), "SHOULD_NOT_EXIST".to_string()];
                    }
                    other => {
                        eprintln!("Unknown compat profile: {}", other);
                        std::process::exit(2);
                    }
                }
            }
            "--strict" => {
                opts.number_tolerance_leading_dot = false;
                opts.number_tolerance_trailing_dot = false;
                opts.number_tolerance_incomplete_exponent = false;
                opts.allow_python_keywords = false;
                opts.repair_undefined = false;
                opts.normalize_js_nonfinite = false;
            }
            "--tolerate-leading-dot" => {
                opts.number_tolerance_leading_dot = true;
            }
            "--no-tolerate-leading-dot" => {
                opts.number_tolerance_leading_dot = false;
            }
            "--tolerate-trailing-dot" => {
                opts.number_tolerance_trailing_dot = true;
            }
            "--no-tolerate-trailing-dot" => {
                opts.number_tolerance_trailing_dot = false;
            }
            "--tolerate-incomplete-exponent" => {
                opts.number_tolerance_incomplete_exponent = true;
            }
            "--no-tolerate-incomplete-exponent" => {
                opts.number_tolerance_incomplete_exponent = false;
            }
            "--word-comment" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Missing MARKER for --word-comment");
                    std::process::exit(2);
                }
                opts.word_comment_markers.push(args[i].clone());
            }
            s if s.starts_with('-') => {
                eprintln!("Unknown option: {}", s);
                std::process::exit(2);
            }
            path => {
                input = Some(path.to_string());
            }
        }
        i += 1;
    }

    // in-place implies non-streaming
    if in_place {
        stream = false;
    }

    let mode = CliMode {
        input,
        output,
        in_place,
        stream,
        chunk_size,
        pretty,
    };
    (opts, mode)
}

struct CliMode {
    input: Option<String>,
    output: Option<String>,
    in_place: bool,
    stream: bool,
    chunk_size: usize,
    pretty: bool,
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let (opts, mode) = parse_args();

    // Resolve IO targets
    let mut out_writer: Box<dyn Write> = if let Some(ref o) = mode.output {
        Box::new(BufWriter::new(File::create(o)?))
    } else {
        Box::new(BufWriter::new(io::stdout()))
    };

    if mode.in_place {
        let inp = mode
            .input
            .as_ref()
            .ok_or("--in-place requires INPUT file")?;
        let content = fs::read_to_string(inp)?;
        let s = repair_to_string(&content, &opts)?;
        if mode.pretty {
            #[cfg(feature = "serde")]
            {
                let v: serde_json::Value = serde_json::from_str(&s)
                    .map_err(|e| crate::RepairError::from_serde("parse", e))?;
                let pretty = serde_json::to_string_pretty(&v)?;
                fs::write(inp, pretty)?;
                return Ok(());
            }
            #[cfg(not(feature = "serde"))]
            {
                fs::write(inp, s)?;
                return Ok(());
            }
        }
        fs::write(inp, s)?;
        return Ok(());
    }

    match (mode.stream, &mode.input) {
        (true, None) => {
            // stdin -> writer (streaming via StreamRepairer)
            let mut r = StreamRepairer::new(opts.clone());
            let mut buf = vec![0u8; mode.chunk_size.max(1024)];
            let mut stdin = io::stdin();
            loop {
                let n = stdin.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                let chunk = std::str::from_utf8(&buf[..n]).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "stdin is not valid UTF-8")
                })?;
                r.push_to_writer(chunk, &mut out_writer)?;
            }
            r.flush_to_writer(&mut out_writer)?;
        }
        (true, Some(path)) => {
            // file -> writer (streaming via StreamRepairer)
            let file = File::open(path)?;
            let mut reader = BufReader::new(file);
            let mut r = StreamRepairer::new(opts.clone());
            let mut buf = vec![0u8; mode.chunk_size.max(1024)];
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                let chunk = std::str::from_utf8(&buf[..n]).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "input is not valid UTF-8")
                })?;
                r.push_to_writer(chunk, &mut out_writer)?;
            }
            r.flush_to_writer(&mut out_writer)?;
        }
        (false, None) => {
            // stdin -> writer (non-streaming: read all, optional pretty)
            let mut content = String::new();
            io::stdin().read_to_string(&mut content)?;
            if mode.pretty {
                let s = repair_to_string(&content, &opts)?;
                #[cfg(feature = "serde")]
                {
                    let v: serde_json::Value = serde_json::from_str(&s)
                        .map_err(|e| crate::RepairError::from_serde("parse", e))?;
                    let pretty = serde_json::to_string_pretty(&v)?;
                    out_writer.write_all(pretty.as_bytes())?;
                }
                #[cfg(not(feature = "serde"))]
                {
                    out_writer.write_all(s.as_bytes())?;
                }
                return Ok(());
            } else {
                repair_to_writer_streaming(&content, &opts, &mut out_writer)?;
            }
        }
        (false, Some(path)) => {
            // file -> writer (non-streaming or streaming-writer as optimization)
            let content = fs::read_to_string(path)?;
            if mode.pretty {
                let s = repair_to_string(&content, &opts)?;
                #[cfg(feature = "serde")]
                {
                    let v: serde_json::Value = serde_json::from_str(&s)
                        .map_err(|e| crate::RepairError::from_serde("parse", e))?;
                    let pretty = serde_json::to_string_pretty(&v)?;
                    out_writer.write_all(pretty.as_bytes())?;
                }
                #[cfg(not(feature = "serde"))]
                {
                    out_writer.write_all(s.as_bytes())?;
                }
                return Ok(());
            } else {
                // stream while parsing to lower peak memory
                repair_to_writer_streaming(&content, &opts, &mut out_writer)?;
            }
        }
    }

    Ok(())
}
