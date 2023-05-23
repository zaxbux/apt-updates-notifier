mod apt;
mod cli;
mod config;
mod error;
mod mail;
mod util;

use error::Result;

//fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
fn main() -> Result<()> {
    let cli = cli::parse();

    match &cli.command {
        Some(command) => match command {
            cli::Command::Configure => cli::commands::configure(&cli),
        },
        None => {
            let config = config::Config::from_file(&cli.config)?;

            let output = apt::update()?;

            let packages = apt::list_upgradeable()?;

            let message = mail::build_message(&config, &packages, &output)?;
            let response = mail::send_smtp(&config, message)?;

            if !response.is_positive() {
                println!("Response: {:?}", response);
            }

            Ok(())
        }
    }
}
