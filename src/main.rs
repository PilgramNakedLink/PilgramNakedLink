use anyhow::{Context, Error, Result};
use dotenv::dotenv;
use pico_args::Arguments;
use std::{
    default::Default,
    net::{AddrParseError, IpAddr},
    path::PathBuf,
    process::exit,
};

mod cmd;

#[derive(Debug)]
pub struct AppConfig {
    pub destination: Option<IpAddr>,
    pub count: i32,
    pub fails: i32,
    pub db: PathBuf,
}

impl AppConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            destination: None,
            count: 1,
            fails: 10,
            db: PathBuf::from("tracer.db"),
        }
    }
}

#[derive(Debug)]
enum AppCommand {
    Init,
    Trace,
    Export,
}

#[derive(Debug)]
struct AppArgs {
    help: bool,
    command: AppCommand,
    cfg: AppConfig,
}

impl AppArgs {
    fn new() -> Self {
        Self {
            help: true,
            command: AppCommand::Trace,
            cfg: AppConfig::new(),
        }
    }
}

const HELP: &str = r#"
Trace routes.

USAGE:
    tracer SUBCOMMAND [OPTIONS] DESTINATION

SUBCOMMANDS:
    init
    trace
    export

OPTIONS:
    -c, --count NUMBER            Number of traces to the destination. Defaults
                                  to 1.
    -n, --num-fails NUMBER        Number of failure for any hop along the way
                                  before giving up. Defaults to 1.
    -D, --db PATH                 Path to SQLITE database. Defaults to ./tracer.db.
    -h, --help                    Prints help information.
"#;

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}

fn parse_ip(s: &str) -> Result<IpAddr, AddrParseError> {
    s.parse()
}

fn main() -> Result<()> {
    // Parse the command line arguments and exit early if we have an issue
    // during parsing or we detected the help flag.
    let args = cli_args()
        .context("Failed to parse application arguments.")
        .unwrap_or_else(|e| {
            eprintln!("{:?}", e);
            exit(1);
        });

    if let true = args.help {
        println!("{}", HELP);
        exit(0);
    }

    // Load the environment.
    dotenv().ok();

    // Run the
    match args.command {
        AppCommand::Init => cmd::init(args.cfg)?,
        AppCommand::Trace => cmd::trace(args.cfg)?,
        AppCommand::Export => cmd::export(args.cfg)?,
    };

    Ok(())
}

fn cli_args() -> Result<AppArgs> {
    // `from_vec` takes `OsString`, not `String`.
    let mut args: Vec<_> = std::env::args_os().collect();
    args.remove(0); // remove the executable path.

    // Find and process `--`.
    let forwarded_args = if let Some(dash_dash) = args.iter().position(|arg| arg == "--") {
        // Store all arguments following ...
        let later_args = args.drain(dash_dash + 1..).collect();
        // .. then remove the `--`
        args.pop();
        later_args
    } else {
        args
    };

    // Now process all forwarded args
    let mut args = Arguments::from_vec(forwarded_args);
    let mut app_args = AppArgs::new();

    // We short circuit the argument parsing if we encounter the help flag. The
    // caller can check for this special case.
    app_args.help = args.contains(["-h", "--help"]);
    if app_args.help {
        return Ok(app_args);
    }

    // We parse required arguments next.
    // .as_deref() requires Rust 1.40: https://stackoverflow.com/a/65423781
    let command = match args.subcommand()?.as_deref() {
        Some("init") => Ok(AppCommand::Init),
        Some("trace") => Ok(AppCommand::Trace),
        Some("export") => Ok(AppCommand::Export),
        Some(v) => Err(Error::msg(format!("{:?} is an invalid command", v))),
        None => Err(Error::msg("missing subcommand")),
    }?;

    app_args.command = command;
    app_args.cfg.destination = args.opt_free_from_fn(parse_ip)?;

    // And now we parse optional arguments.
    if let Some(count) = args.opt_value_from_str(["-c", "--count"])? {
        app_args.cfg.count = count;
    }

    if let Some(fails) = args.opt_value_from_str(["-n", "--num-fails"])? {
        app_args.cfg.fails = fails;
    }

    if let Ok(db) = args.value_from_os_str(["-D", "--db"], parse_path) {
        app_args.cfg.db = db;
    }

    Ok(app_args)
}
