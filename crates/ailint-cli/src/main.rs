//! The `ailint` binary.
//!
//! Subcommands: `check`, `stats`, `init`, `list-rules`. Everything is a
//! scaffold — see `TODO` markers.

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use ailint_core::config::{ColorMode, Config};
use ailint_core::reporter::{self, ReporterKind};

/// Lint and inspect AI agent guidance files.
#[derive(Debug, Parser)]
#[command(name = "ailint", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Path to config file (default: auto-discover `.ailint.yaml`).
    #[arg(long, global = true, value_name = "PATH", env = "AILINT_CONFIG")]
    config: Option<PathBuf>,

    /// Increase logging verbosity (`-v`, `-vv`).
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Lint guidance files in PATH (default: current directory).
    Check(CheckArgs),
    /// Print coverage / stats about guidance files.
    Stats(StatsArgs),
    /// Scaffold a `.ailint.yaml` in the current directory.
    Init,
    /// List every rule with its ID, slug, and default severity.
    ListRules,
}

#[derive(Debug, clap::Args)]
struct CheckArgs {
    /// Path(s) to lint. Defaults to the current directory.
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = FormatArg::Terminal)]
    format: FormatArg,

    /// Write output to FILE instead of stdout.
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Enable only the given rule (repeatable). ID or slug.
    #[arg(long, value_name = "ID")]
    rule: Vec<String>,

    /// Disable a rule (repeatable). ID or slug.
    #[arg(long, value_name = "ID")]
    ignore: Vec<String>,

    /// Fail with non-zero exit if more than N warnings are found.
    #[arg(long, value_name = "N")]
    max_warnings: Option<usize>,

    /// LLM provider for AI-powered rules (opt-in).
    #[arg(long, value_enum, value_name = "PROVIDER")]
    llm_provider: Option<LlmProviderArg>,

    /// Model name for the chosen LLM provider.
    #[arg(long, value_name = "MODEL", requires = "llm_provider")]
    llm_model: Option<String>,
}

#[derive(Debug, clap::Args)]
struct StatsArgs {
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum FormatArg {
    Terminal,
    Json,
    Sarif,
    Markdown,
}

impl From<FormatArg> for ReporterKind {
    fn from(f: FormatArg) -> Self {
        match f {
            FormatArg::Terminal => ReporterKind::Terminal,
            FormatArg::Json => ReporterKind::Json,
            FormatArg::Sarif => ReporterKind::Sarif,
            FormatArg::Markdown => ReporterKind::Markdown,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LlmProviderArg {
    Openai,
    Anthropic,
    Google,
    Ollama,
    /// Any OpenAI-compatible endpoint.
    Compatible,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("ailint: error: {e:#}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    // TODO: proper config discovery — walk up from cwd.
    let config = if let Some(ref p) = cli.config {
        Config::load(p)?
    } else {
        Config::default()
    };

    match cli.command {
        Command::Check(args) => cmd_check(args, config),
        Command::Stats(args) => cmd_stats(args, config),
        Command::Init => cmd_init(),
        Command::ListRules => cmd_list_rules(),
    }
}

fn cmd_check(args: CheckArgs, mut config: Config) -> Result<ExitCode> {
    // TODO: apply --rule / --ignore / --max-warnings / --llm-* overrides on
    // top of the loaded config before running.
    for r in &args.ignore {
        config.rules.disabled.push(r.clone());
    }
    let _ = &args.rule;
    let _ = &args.max_warnings;
    let _ = &args.llm_provider;
    let _ = &args.llm_model;

    let mut violations = Vec::new();
    for path in &args.paths {
        violations.extend(ailint_core::lint(path, &config)?);
    }

    let reporter = reporter::make(args.format.into());
    let mut sink: Box<dyn Write> = match args.output {
        Some(ref p) => Box::new(std::fs::File::create(p)?),
        None => Box::new(io::stdout()),
    };
    reporter.report(&violations, sink.as_mut())?;

    // TODO: real exit-code policy — fail on any Severity::Error, respect
    // --max-warnings, etc.
    if violations
        .iter()
        .any(|v| matches!(v.severity, ailint_core::Severity::Error))
    {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn cmd_stats(_args: StatsArgs, _config: Config) -> Result<ExitCode> {
    // TODO: file counts by FileType, rule density, avg word count per rule,
    // top-N files by violation count.
    println!("ailint: stats not yet implemented");
    Ok(ExitCode::SUCCESS)
}

fn cmd_init() -> Result<ExitCode> {
    // TODO: refuse to overwrite an existing .ailint.yaml unless --force is
    // passed. Emit a fully-commented template.
    const TEMPLATE: &str = include_str!("../../../.ailint.yaml.template");
    let path = std::path::Path::new(".ailint.yaml");
    if path.exists() {
        eprintln!("ailint: .ailint.yaml already exists");
        return Ok(ExitCode::from(1));
    }
    std::fs::write(path, TEMPLATE)?;
    println!("ailint: wrote .ailint.yaml");
    Ok(ExitCode::SUCCESS)
}

fn cmd_list_rules() -> Result<ExitCode> {
    // TODO: iterate `registry::all_rules()` once populated. For now emit the
    // static constants directly so users can see the planned rule surface.
    use ailint_core::rules::{consistency, security, semantic, structural};
    println!("{:<8}  {:<40}  category", "code", "slug");
    println!("{:-<8}  {:-<40}  {:-<12}", "", "", "");
    let rows: &[(ailint_core::RuleId, &str)] = &[
        (structural::AIL001, "structural"),
        (structural::AIL002, "structural"),
        (structural::AIL003, "structural"),
        (semantic::AIL100, "semantic"),
        (semantic::AIL101, "semantic"),
        (semantic::AIL102, "semantic"),
        (semantic::AIL103, "semantic"),
        (security::AIL200, "security"),
        (security::AIL201, "security"),
        (security::AIL202, "security"),
        (consistency::AIL300, "consistency"),
        (consistency::AIL301, "consistency"),
    ];
    for (id, category) in rows {
        println!("{:<8}  {:<40}  {}", id.code_str(), id.slug, category);
    }
    Ok(ExitCode::SUCCESS)
}

fn init_tracing(verbose: u8) {
    let level = match verbose {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };
    // TODO: allow AILINT_LOG env override, structured JSON logging in CI.
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("AILINT_LOG")
                .unwrap_or_else(|_| level.into()),
        )
        .with_target(false)
        .try_init();
}

// Silence unused imports during scaffold.
#[allow(dead_code)]
fn _touch(_c: ColorMode) {}
