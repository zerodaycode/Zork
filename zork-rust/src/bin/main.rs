use env_logger::Builder;
use log::LevelFilter;
use tracing::{info};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};
use zork::{utils::logger::CustomLayer, config_cli::CliArgs};
use clap::Parser;
use std::io::Write;
use chrono::Local;


fn main() {
    let parser_cli = CliArgs::parse_from(["","-vv"]);
    
    tracing_subscriber::registry().with(CustomLayer{
        verbose_level: parser_cli.verbose.clone()
    }).init();
    info!(m="asd",a="aaaa");

    let mut builder = Builder::from_default_env();


    builder
        .format(|buf, record| {
            writeln!(buf,
                "{} [{}] - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .try_init();

    log::warn!("warn");
    log::info!("info");
    log::debug!("debug");
}
