use std::{
    fs::OpenOptions,
    io::{BufWriter, Write},
};

use crate::{
    error::{Error, Result},
    util,
};
use lettre::{
    message::{header, Mailbox, Mailboxes},
    transport::smtp,
};
use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub struct Config {
    pub smtp: SMTP,
    pub mail: Mail,
}

impl Config {
    pub fn from_file(file: &str) -> Result<Config> {
        let config = config::Config::builder()
            .add_source(config::File::with_name(file).format(config::FileFormat::Toml))
            .build()?;

        Ok(config.try_deserialize()?)
    }

    pub fn to_file(&self, file: &str) -> Result<()> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file)
            .map_err(|err| Error::ConfigWrite(err.to_string()))?;
        let mut writer = BufWriter::new(file);
        writer
            .write_all(
                toml::to_string(self)
                    .map_err(|err| Error::ConfigWrite(err.to_string()))?
                    .as_bytes(),
            )
            .map_err(|err| Error::ConfigWrite(err.to_string()))?;
        Ok(())
    }
}

#[derive(Default, Deserialize, Serialize)]
pub struct SMTP {
    relay: String,
    auth_username: String,
    auth_password: String,
}

impl SMTP {
    pub fn new(relay: String, auth_username: String, auth_password: String) -> Self {
        Self {
            relay,
            auth_username,
            auth_password,
        }
    }

    pub fn relay(&self) -> String {
        self.relay.clone()
    }

    pub fn username(&self) -> String {
        self.auth_username.clone()
    }

    pub fn password(&self) -> String {
        self.auth_password.clone()
    }

    pub fn credentials(&self) -> smtp::authentication::Credentials {
        smtp::authentication::Credentials::new(self.username(), self.password())
    }
}

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct Mail {
    pub from: String,
    to: Vec<Mailbox>,
    subject: String,
    html: bool,
    prepend: Option<String>,
    append: Option<String>,
}

impl Mail {
    pub fn new(from: String, to: Vec<Mailbox>, subject: String) -> Self {
        Self {
            from,
            to,
            subject,
            ..Default::default()
        }
    }

    pub fn from(&self) -> Result<Mailbox> {
        let from: Mailbox = self.from.parse().unwrap();

        if from.name.is_none() {
            return Ok(Mailbox::new(util::get_hostname(), from.email));
        }

        Ok(from)
    }
    pub fn to(&self) -> Mailboxes {
        self.to.clone().into()
    }
    pub fn subject_fmt(&self, count: usize) -> String {
        self.subject
            .clone()
            .replace('#', &count.to_string())
            .replace('@', &util::get_hostname().unwrap_or("".to_string()))
    }

    pub fn subject(&self) -> String {
        self.subject.clone()
    }

    pub fn to_header(&self) -> header::To {
        self.to().into()
    }
    pub fn html(&self) -> bool {
        self.html
    }
    pub fn prepend(&self) -> Option<String> {
        self.prepend.clone()
    }
    pub fn append(&self) -> Option<String> {
        self.append.clone()
    }
}

impl Default for Mail {
    fn default() -> Mail {
        Mail {
            subject: "There are # package updates available for @".to_string(),
            html: true,
            from: "".to_string(),
            to: Vec::new(),
            prepend: None,
            append: None,
        }
    }
}
