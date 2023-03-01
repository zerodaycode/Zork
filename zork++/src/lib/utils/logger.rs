use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use env_logger::{Builder, Target};
use log::LevelFilter;

#[allow(clippy::comparison_chain)]
/// [`config_logger`] The configuration for `env_logger`
pub fn config_logger(verbose_level: u8, target: Target) -> Result<()> {
    let mut builder = Builder::from_default_env();

    builder
        .target(target)
        // .default_format()
        .format_indent(Some(4))
        .format_module_path(false)
        .format_timestamp_millis();
    // .write_style(env_logger::WriteStyle::Always);

    if verbose_level == 1 {
        builder.filter(None, LevelFilter::Debug);
    } else if verbose_level > 1 {
        return Err(eyre!("Zork++ maximum allowed verbosity level is: '-v'"));
    } else {
        builder.filter(None, LevelFilter::Info);
    }

    builder
        .try_init()
        .with_context(|| "Zork++ wasn't unable to set up the logger")
}
