use env_logger::{Builder, Target};
use log::{LevelFilter, SetLoggerError};

///
/// [`config_logger`] configure logger
///
/// TODO to capture output and make test required top layer
///
pub fn config_logger(verbose_level: u8, target: Target) -> Result<(), SetLoggerError> {
    let mut builder = Builder::from_default_env();

    builder
        .target(target)
        .default_format()
        .write_style(env_logger::WriteStyle::Always);

    if verbose_level == 1 {
        builder.filter(None, LevelFilter::Warn);
    } else if verbose_level == 2 {
        builder.filter(None, LevelFilter::Info);
    } else if verbose_level > 2 {
        panic!("Zork++ not support this verbose level, max level 2 '-vv'")
    } else {
        builder.filter(None, LevelFilter::Error);
    }

    builder.try_init()
}
