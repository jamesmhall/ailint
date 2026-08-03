//! The `ailint` binary. Subcommands: `check`, `stats`, `init`, `list-rules`,
//! `schema`.

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand, ValueEnum};

use ailint_core::config::{ColorMode, Config, LlmConfig, LlmProviderKind};
use ailint_core::reporter::{self, ReporterKind};
use ailint_core::rules::{registry, Severity};

mod stats;

#[cfg(feature = "llm")]
mod llm_bridge;

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

    /// Log output format.
    #[arg(long, global = true, value_enum, default_value_t = LogFormatArg::Text)]
    log_format: LogFormatArg,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Lint guidance files in PATH (default: current directory).
    Check(CheckArgs),
    /// Print coverage / stats about guidance files.
    Stats(StatsArgs),
    /// Scaffold a `.ailint.yaml` in the current directory.
    Init(InitArgs),
    /// List every rule with its ID, slug, and default severity.
    ListRules,
    /// Print the JSON Schema for `.ailint.yaml`.
    Schema,
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

    /// Apply deterministic auto-fixes for any fixable violation, in place.
    /// Files with overlapping fix ranges are skipped and reported as
    /// conflicts.
    #[arg(long)]
    fix: bool,
}

#[derive(Debug, clap::Args)]
struct StatsArgs {
    #[arg(value_name = "PATH", default_value = ".")]
    paths: Vec<PathBuf>,
}

#[derive(Debug, clap::Args)]
struct InitArgs {
    /// Overwrite an existing .ailint.yaml.
    #[arg(short, long)]
    force: bool,
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

impl From<LlmProviderArg> for LlmProviderKind {
    fn from(p: LlmProviderArg) -> Self {
        match p {
            LlmProviderArg::Openai => LlmProviderKind::Openai,
            LlmProviderArg::Anthropic => LlmProviderKind::Anthropic,
            LlmProviderArg::Google => LlmProviderKind::Google,
            LlmProviderArg::Ollama => LlmProviderKind::Ollama,
            LlmProviderArg::Compatible => LlmProviderKind::Compatible,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum LogFormatArg {
    #[default]
    Text,
    Json,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    init_tracing(cli.verbose, cli.log_format);

    match run(cli) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("ailint: error: {e:#}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    let config = resolve_config(&cli)?;

    match cli.command {
        Command::Check(args) => cmd_check(args, config),
        Command::Stats(args) => cmd_stats(args, config),
        Command::Init(args) => cmd_init(args),
        Command::ListRules => cmd_list_rules(),
        Command::Schema => cmd_schema(),
    }
}

fn resolve_config(cli: &Cli) -> Result<Config> {
    if let Some(ref p) = cli.config {
        tracing::info!(path = %p.display(), "loading config");
        return Config::load(p);
    }

    let start: PathBuf = match &cli.command {
        Command::Check(a) => a
            .paths
            .first()
            .cloned()
            .unwrap_or_else(|| PathBuf::from(".")),
        Command::Stats(a) => a
            .paths
            .first()
            .cloned()
            .unwrap_or_else(|| PathBuf::from(".")),
        Command::Init(_) | Command::ListRules | Command::Schema => PathBuf::from("."),
    };

    if let Some(found) = Config::discover(&start) {
        tracing::info!(path = %found.display(), "discovered config");
        Config::load(&found)
    } else {
        tracing::info!("using defaults (no config found)");
        Ok(Config::default())
    }
}

fn cmd_check(args: CheckArgs, mut config: Config) -> Result<ExitCode> {
    apply_check_overrides(&args, &mut config)?;

    let mut violations = Vec::new();
    for path in &args.paths {
        violations.extend(ailint_core::lint(path, &config)?);
        #[cfg(feature = "llm")]
        {
            violations.extend(llm_bridge::run(&config, path));
        }
    }

    let fix_summary = if args.fix {
        Some(apply_fixes_and_report(&violations)?)
    } else {
        None
    };
    // After --fix, re-run so the reporter shows what's actually left.
    if fix_summary.as_ref().is_some_and(|s| s.applied > 0) {
        violations.clear();
        for path in &args.paths {
            violations.extend(ailint_core::lint(path, &config)?);
            #[cfg(feature = "llm")]
            {
                violations.extend(llm_bridge::run(&config, path));
            }
        }
    }

    let reporter = reporter::make(args.format.into());
    let mut sink: Box<dyn Write> = match args.output {
        Some(ref p) => Box::new(std::fs::File::create(p)?),
        None => Box::new(io::stdout()),
    };
    reporter.report(&violations, sink.as_mut())?;

    let error_count = violations
        .iter()
        .filter(|v| matches!(v.severity, Severity::Error))
        .count();
    let warning_count = violations
        .iter()
        .filter(|v| matches!(v.severity, Severity::Warning))
        .count();
    if let Some(summary) = fix_summary {
        // Conflicts are user-actionable: we did not touch those files.
        if summary.conflicts > 0 {
            return Ok(ExitCode::from(1));
        }
    }
    if error_count > 0 {
        return Ok(ExitCode::from(1));
    }
    if let Some(max) = args.max_warnings {
        if warning_count > max {
            return Ok(ExitCode::from(1));
        }
    }
    Ok(ExitCode::SUCCESS)
}

struct FixSummary {
    applied: usize,
    conflicts: usize,
}

fn apply_fixes_and_report(violations: &[ailint_core::Violation]) -> Result<FixSummary> {
    let results = ailint_core::apply_fixes(violations)?;
    let mut applied = 0usize;
    let mut conflicts = 0usize;
    for r in &results {
        if let Some(c) = &r.conflict {
            conflicts += 1;
            eprintln!(
                "ailint: fix conflict in {}: edits {:?} and {:?} overlap; file left unchanged",
                r.path.display(),
                c.first,
                c.second
            );
        } else if r.applied > 0 {
            applied += r.applied;
            eprintln!(
                "ailint: fixed {} in {} ({} edits)",
                if r.applied == 1 { "1 issue" } else { "issues" },
                r.path.display(),
                r.applied
            );
        }
    }
    Ok(FixSummary { applied, conflicts })
}

fn apply_check_overrides(args: &CheckArgs, config: &mut Config) -> Result<()> {
    for r in &args.ignore {
        config.rules.disabled.push(r.clone());
    }

    if !args.rule.is_empty() {
        let allow: std::collections::HashSet<&str> = args.rule.iter().map(|s| s.as_str()).collect();
        for rule in registry::all_rules() {
            let id = rule.id();
            let code = id.code_str();
            if !allow.contains(code.as_str()) && !allow.contains(id.slug) {
                config.rules.disabled.push(code);
            }
        }
        for rule in registry::all_batch_rules() {
            let id = rule.id();
            let code = id.code_str();
            if !allow.contains(code.as_str()) && !allow.contains(id.slug) {
                config.rules.disabled.push(code);
            }
        }
    }

    match (args.llm_provider, args.llm_model.clone()) {
        (Some(p), Some(m)) => {
            if let Some(existing) = config.llm.as_mut() {
                existing.provider = p.into();
                existing.model = m;
            } else {
                config.llm = Some(LlmConfig {
                    provider: p.into(),
                    model: m,
                    base_url: None,
                    timeout_seconds: None,
                    max_tokens: None,
                    temperature: None,
                    cost_cap_usd: None,
                });
            }
        }
        (Some(p), None) => {
            if let Some(existing) = config.llm.as_mut() {
                existing.provider = p.into();
            } else {
                return Err(anyhow!(
                    "--llm-provider requires --llm-model when no model is set in config"
                ));
            }
        }
        (None, _) => {}
    }

    Ok(())
}

fn cmd_stats(args: StatsArgs, config: Config) -> Result<ExitCode> {
    stats::run(&args.paths, &config)
}

fn cmd_init(args: InitArgs) -> Result<ExitCode> {
    const TEMPLATE: &str = include_str!("../.ailint.yaml.template");
    let path = Path::new(".ailint.yaml");
    if path.exists() && !args.force {
        eprintln!("ailint: .ailint.yaml already exists (pass --force to overwrite)");
        return Ok(ExitCode::from(1));
    }
    std::fs::write(path, TEMPLATE)?;
    println!("ailint: wrote .ailint.yaml");
    Ok(ExitCode::SUCCESS)
}

fn cmd_list_rules() -> Result<ExitCode> {
    println!("{:<8}  {:<40}  {:<12}  default", "code", "slug", "category");
    println!("{:-<8}  {:-<40}  {:-<12}  {:-<8}", "", "", "", "");
    for rule in registry::all_rules() {
        let id = rule.id();
        println!(
            "{:<8}  {:<40}  {:<12}  {}",
            id.code_str(),
            id.slug,
            category_for(id.code),
            rule.default_severity().as_str(),
        );
    }
    for rule in registry::all_batch_rules() {
        let id = rule.id();
        println!(
            "{:<8}  {:<40}  {:<12}  {}",
            id.code_str(),
            id.slug,
            category_for(id.code),
            rule.default_severity().as_str(),
        );
    }
    #[cfg(feature = "llm")]
    {
        println!();
        println!("LLM rules (opt-in):");
        for id in [ailint_llm::AIL900, ailint_llm::AIL901] {
            println!(
                "{:<8}  {:<40}  {:<12}  {}",
                id.code_str(),
                id.slug,
                category_for(id.code),
                Severity::Info.as_str(),
            );
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn cmd_schema() -> Result<ExitCode> {
    println!("{}", Config::json_schema()?);
    Ok(ExitCode::SUCCESS)
}

fn category_for(code: u16) -> &'static str {
    match code {
        1..=99 => "structural",
        100..=199 => "semantic",
        200..=299 => "security",
        300..=399 => "consistency",
        900..=999 => "llm",
        _ => "other",
    }
}

fn init_tracing(verbose: u8, log_format: LogFormatArg) {
    let level = match verbose {
        0 => "warn",
        1 => "info",
        _ => "debug",
    };
    let filter =
        tracing_subscriber::EnvFilter::try_from_env("AILINT_LOG").unwrap_or_else(|_| level.into());
    match log_format {
        LogFormatArg::Text => {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_target(false)
                .try_init();
        }
        LogFormatArg::Json => {
            let _ = tracing_subscriber::fmt()
                .with_env_filter(filter)
                .with_target(false)
                .json()
                .try_init();
        }
    }
}

// Silence unused imports during scaffold.
#[allow(dead_code)]
fn _touch(_c: ColorMode) {}
