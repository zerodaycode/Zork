use env_logger::{Builder, Target};
use log::LevelFilter;
use std::io::Write;
use color_eyre::{Result, eyre::{eyre, Context}};

/// [`config_logger`] The configuration for `env_logger`
pub fn config_logger(verbose_level: u8, target: Target) -> Result<()> {
    let mut builder = Builder::from_default_env();

    builder
        .target(target)
        .format(|buf, record| {
            writeln!(buf, "[{:?} - {}]: {}", chrono::offset::Local::now(), record.level(), record.args())
        })
        .write_style(env_logger::WriteStyle::Always);

    if verbose_level == 1 {
        builder.filter(None, LevelFilter::Debug);
    } else if verbose_level > 1 {
        return Err(eyre!("Zork++ maximum allowed verbosity level is: '-v'"));
    } else {
        builder.filter(None, LevelFilter::Info);
    }

    builder.try_init().with_context(|| "Zork++ wasn't unable to set up the logger")
}
