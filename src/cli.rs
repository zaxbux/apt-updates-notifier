use clap::{Parser, Subcommand};

/// Sends notification containing upgradeable packages.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Config file
    #[arg(short, long, default_value = "/etc/pkg-updates-notifier.conf")]
    pub config: String,

    #[clap(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Set config
    Configure,
}

pub(crate) fn parse() -> Cli {
    Cli::parse()
}

pub mod commands {
    use super::Cli;
    use crate::Result;
    use dialoguer::{theme::ColorfulTheme, Input, Password};
    use lettre::message::Mailbox;

    pub fn configure(cli: &Cli) -> Result<()> {
        let config = crate::config::Config::from_file(&cli.config).unwrap_or_default();

        let smtp_relay: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("[smtp] relay")
            .with_initial_text(config.smtp.relay())
            .interact_text()
            .unwrap();

        let smtp_username: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("[smtp] username")
            .with_initial_text(config.smtp.username())
            .interact_text()
            .unwrap();

        let smtp_password: String = Password::with_theme(&ColorfulTheme::default())
            .with_prompt("[smtp] password")
            .allow_empty_password(true)
            .interact()
            .unwrap();
    
        let smtp_password = if smtp_password.is_empty() {
            config.smtp.password()
        } else {
            smtp_password
        };

        let mail_from: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("[mail] from")
            .with_initial_text(config.mail.from.clone())
            .validate_with(|input: &String| -> std::result::Result<(), String> {
                if let Err(err) = input.parse::<Mailbox>() {
                    return Err(format!("Invalid address: {}", err.to_string()));
                }
                Ok(())
            })
            .interact_text()
            .unwrap();

        let mail_to: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("[mail] to")
            .with_initial_text(config.mail.to().to_string())
            .validate_with(|input: &String| -> std::result::Result<(), String> {
                if let Err(err) = input.parse::<Mailbox>() {
                    return Err(format!("Invalid address: {}", err.to_string()));
                }
                Ok(())
            })
            .interact_text()
            .unwrap();

        let mail_to: Mailbox = mail_to.parse().unwrap();

        let mail_subject: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("[mail] subject")
            .default(crate::config::Mail::default().subject())
            .with_initial_text(config.mail.subject())
            .interact_text()
            .unwrap();

        let config = crate::config::Config {
            smtp: crate::config::SMTP::new(smtp_relay, smtp_username, smtp_password),
            mail: crate::config::Mail::new(mail_from, vec![mail_to], mail_subject),
        };

        config.to_file(&cli.config)?;

        Ok(())
    }
}
