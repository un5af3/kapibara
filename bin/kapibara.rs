//! Kapibara main program
use std::{fs, path::PathBuf};

use clap::Parser;
use env_logger::Env;
use kapibara::{Codec, Dispatch, DispatchOption};
use tokio::signal;

#[derive(Debug, Parser)]
#[command(version)]
#[command(about = "Kapibara~~")]
struct Cli {
    #[arg(short = 'c', long, value_name = "FILE", default_value = "config.yaml")]
    config: PathBuf,
}

fn init_logger() {
    env_logger::Builder::from_env(Env::default())
        .filter_level(log::LevelFilter::Info)
        .init();
}

#[tokio::main]
async fn main() {
    init_logger();

    let cli = Cli::parse();

    let opt_str = match fs::read_to_string(&cli.config) {
        Ok(o) => o,
        Err(e) => {
            log::error!("read file error {}", e);
            return;
        }
    };

    let dispatch_option: DispatchOption = {
        if let Ok(opt) = Codec::Json.from_str(&opt_str) {
            opt
        } else if let Ok(opt) = Codec::Yaml.from_str(&opt_str) {
            opt
        } else {
            log::error!("unsupport config file {}", cli.config.to_string_lossy());
            return;
        }
    };

    let mut dispatcher = match Dispatch::init(dispatch_option) {
        Ok(v) => v,
        Err(e) => {
            log::error!("{}", e);
            return;
        }
    };

    if let Err(e) = dispatcher.start() {
        log::error!("{}", e);
        return;
    }

    let _ = signal::ctrl_c().await;

    dispatcher.close();
}
