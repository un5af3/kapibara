//! Kapibara main program
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand, ValueEnum};
use env_logger::Env;
use tokio::{fs, signal};

use kapibara::{Codec, Dispatch, DispatchOption};

#[derive(Debug, Parser)]
#[command(version)]
#[command(about = "Kapibara~~")]
struct Cli {
    /// Log level
    #[arg(long, value_enum, default_value_t = LogLevel::Info, value_name = "LEVEL")]
    log: LogLevel,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run app
    Run {
        #[arg(short = 'c', long, value_name = "FILE", default_value = "config.yaml")]
        config: PathBuf,
    },
    /// Test config
    Test {
        #[arg(short = 'c', long, value_name = "FILE", default_value = "config.yaml")]
        config: PathBuf,
    },
    /// Generate something
    #[command(subcommand)]
    Gen(Generate),
}

#[derive(Debug, Subcommand)]
enum Generate {
    /// generate uuid
    Uuid,
    /// generate self signed
    Cert {
        #[arg(short, long, value_delimiter = ',')]
        domain: Vec<String>,
    },
}

fn init_logger(level: LogLevel) {
    env_logger::Builder::from_env(Env::default())
        .filter_level(level.into())
        .init();
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    init_logger(cli.log);

    match cli.command {
        Commands::Run { config } => {
            if let Err(err) = run(config).await {
                log::error!("[main::run] {}", err);
            }
        }
        Commands::Test { config } => {
            if let Err(err) = test(config).await {
                log::error!("[main::test] {}", err);
            }
        }
        Commands::Gen(gen) => match gen {
            Generate::Uuid => {
                let uuid = uuid::Uuid::new_v4();
                println!("{}", uuid);
            }
            Generate::Cert { domain } => {
                if let Err(err) = gen_cert(domain).await {
                    log::error!("[main::gen::cert] {}", err);
                }
            }
        },
    }
}

async fn run(config: PathBuf) -> Result<()> {
    let dispatch_option = parse_config(&config).await?;
    let mut dispatcher = Dispatch::init(dispatch_option)?;

    dispatcher.start()?;

    let _ = signal::ctrl_c().await;

    dispatcher.close();

    Ok(())
}

async fn test(config: PathBuf) -> Result<()> {
    let opt = parse_config(&config).await?;

    let pretty_str = serde_json::to_string_pretty(&opt)?;

    println!("{}", pretty_str);

    Ok(())
}

async fn gen_cert(domain: Vec<String>) -> Result<()> {
    let rcgen::CertifiedKey { cert, key_pair } = rcgen::generate_simple_self_signed(domain)?;

    println!("{}", cert.pem());
    println!("{}", key_pair.serialize_pem());

    Ok(())
}

async fn parse_config(config: &PathBuf) -> Result<DispatchOption> {
    let opt_str = fs::read_to_string(config).await?;

    if let Some(ext) = config.extension() {
        match ext.to_ascii_lowercase().to_string_lossy().as_ref() {
            "yaml" => return Ok(Codec::Yaml.from_str(&opt_str)?),
            "json" => return Ok(Codec::Json.from_str(&opt_str)?),
            other => log::warn!("invalid file extension {}", other),
        }
    }

    if let Ok(opt) = Codec::Yaml.from_str(&opt_str) {
        return Ok(opt);
    } else if let Ok(opt) = Codec::Json.from_str(&opt_str) {
        return Ok(opt);
    }

    Err(anyhow!("invalid config file: {}", config.to_string_lossy()))
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Trace => Self::Trace,
            LogLevel::Debug => Self::Debug,
            LogLevel::Info => Self::Info,
            LogLevel::Warn => Self::Warn,
            LogLevel::Error => Self::Error,
            LogLevel::Off => Self::Off,
        }
    }
}
