//! CLI argument parsing

use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Args {
  pub input: PathBuf,
  pub output: PathBuf,
  pub format: OutputFormat,
  pub recursive: bool,
  pub verbose: bool,
  pub parallel: bool,
  pub pretty: bool,
  pub validate: bool,
  pub sourcemap: bool,
  pub bench: bool,
  pub streaming: bool,
  pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
  Dast,
  Json,
}

impl Default for Args {
  fn default() -> Self {
    Self {
      input: PathBuf::from("."),
      output: PathBuf::from("./ast_output"),
      format: OutputFormat::Dast,
      recursive: true,
      verbose: false,
      parallel: true,
      pretty: false,
      validate: false,
      sourcemap: false,
      bench: false,
      streaming: false,
      extensions: vec![
        "md".to_string(),
        "markdown".to_string(),
        "js".to_string(),
        "mjs".to_string(),
        "cjs".to_string(),
        "ts".to_string(),
        "tsx".to_string(),
        "mts".to_string(),
        "java".to_string(),
        "py".to_string(),
        "pyi".to_string(),
      ],
    }
  }
}

pub fn parse_args() -> Result<Args, String> {
  let args: Vec<String> = env::args().collect();

  if args.len() < 2 {
    return Err(get_help());
  }

  let mut result = Args::default();
  let mut i = 1;

  while i < args.len() {
    match args[i].as_str() {
      "-h" | "--help" => {
        return Err(get_help());
      }
      "-v" | "--version" => {
        return Err("bukvar v1.0.0 (Glagolica Project)".to_string());
      }
      "-i" | "--input" => {
        i += 1;
        if i >= args.len() {
          return Err("Missing argument for --input".to_string());
        }
        result.input = PathBuf::from(&args[i]);
      }
      "-o" | "--output" => {
        i += 1;
        if i >= args.len() {
          return Err("Missing argument for --output".to_string());
        }
        result.output = PathBuf::from(&args[i]);
      }
      "-f" | "--format" => {
        i += 1;
        if i >= args.len() {
          return Err("Missing argument for --format".to_string());
        }
        result.format = match args[i].to_lowercase().as_str() {
          "dast" | "binary" => OutputFormat::Dast,
          "json" => OutputFormat::Json,
          _ => return Err(format!("Unknown format: {}. Use 'dast' or 'json'", args[i])),
        };
      }
      "-e" | "--ext" | "--extensions" => {
        i += 1;
        if i >= args.len() {
          return Err("Missing argument for --extensions".to_string());
        }
        result.extensions = args[i].split(',').map(|s| s.trim().to_string()).collect();
      }
      "--no-recursive" => {
        result.recursive = false;
      }
      "-r" | "--recursive" => {
        result.recursive = true;
      }
      "--verbose" => {
        result.verbose = true;
      }
      "--no-parallel" => {
        result.parallel = false;
      }
      "--pretty" => {
        result.pretty = true;
      }
      "--validate" => {
        result.validate = true;
      }
      "--sourcemap" => {
        result.sourcemap = true;
      }
      "--bench" => {
        result.bench = true;
      }
      "--streaming" => {
        result.streaming = true;
      }
      arg if !arg.starts_with('-') => {
        // Positional argument: treat first as input, second as output
        if result.input.as_os_str() == "." {
          result.input = PathBuf::from(arg);
        } else {
          result.output = PathBuf::from(arg);
        }
      }
      _ => {
        return Err(format!("Unknown argument: {}", args[i]));
      }
    }
    i += 1;
  }

  Ok(result)
}

fn get_help() -> String {
  r#"bukvar - Ultra-fast zero-dependency markdown parser (Glagolica Project)

USAGE:
    bukvar [OPTIONS] <INPUT> [OUTPUT]

OPTIONS:
    -i, --input <PATH>      Input directory
    -o, --output <PATH>     Output directory (default: ./ast_output)
    -f, --format <FMT>      dast (binary) or json (default: dast)
    -e, --extensions <EXT>  Comma-separated extensions
    -r, --recursive         Recurse into subdirs (default: on)
    --no-recursive          Don't recurse
    --no-parallel           Single-threaded
    --pretty                Pretty-print JSON output
    --validate              Check for broken links/refs
    --sourcemap             Generate source maps (.map.json)
    --streaming             Use streaming parser for large files
    --bench                 Run internal benchmarks
    --verbose               Show progress
    -h, --help
    -v, --version

EXAMPLES:
    bukvar ./src ./output -f json --pretty
    bukvar -i ./docs -o ./ast --validate --sourcemap
    bukvar -i ./large-docs --streaming
"#
  .to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_default_args() {
    let args = Args::default();
    assert_eq!(args.input, PathBuf::from("."));
    assert_eq!(args.output, PathBuf::from("./ast_output"));
    assert_eq!(args.format, OutputFormat::Dast);
    assert!(args.recursive);
    assert!(!args.verbose);
    assert!(args.parallel);
    assert!(!args.pretty);
    assert!(!args.validate);
    assert!(!args.sourcemap);
    assert!(!args.bench);
    assert!(!args.streaming);
  }

  #[test]
  fn test_output_format_eq() {
    assert_eq!(OutputFormat::Dast, OutputFormat::Dast);
    assert_eq!(OutputFormat::Json, OutputFormat::Json);
    assert_ne!(OutputFormat::Dast, OutputFormat::Json);
  }

  #[test]
  fn test_output_format_debug() {
    assert_eq!(format!("{:?}", OutputFormat::Dast), "Dast");
    assert_eq!(format!("{:?}", OutputFormat::Json), "Json");
  }

  #[test]
  fn test_args_clone() {
    let args = Args::default();
    let cloned = args.clone();
    assert_eq!(args.input, cloned.input);
    assert_eq!(args.format, cloned.format);
  }

  #[test]
  fn test_default_extensions() {
    let args = Args::default();
    assert!(args.extensions.contains(&"md".to_string()));
    assert!(args.extensions.contains(&"js".to_string()));
    assert!(args.extensions.contains(&"py".to_string()));
    assert!(args.extensions.contains(&"java".to_string()));
    assert!(args.extensions.contains(&"ts".to_string()));
  }

  #[test]
  fn test_help_contains_usage() {
    let help = get_help();
    assert!(help.contains("USAGE:"));
    assert!(help.contains("OPTIONS:"));
    assert!(help.contains("EXAMPLES:"));
    assert!(help.contains("bukvar"));
  }
}
