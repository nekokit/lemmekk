use std::env;

use clap::Parser;
use lemmekk::AppError;
use lemmekk::Application;

use lemmekk::CliArgs;

fn main() -> Result<(), AppError> {
    let mut app = Application::create(CliArgs::parse())?;
    env::set_var("RUST_LOG", app.config.general.log_level.to_string());
    env_logger::init();

    app.run()
}
