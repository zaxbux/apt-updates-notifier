use std::{error, fmt, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Config(config::ConfigError),
    ConfigWrite(String),
    AptCache(cxx::Exception),
    Email(lettre::error::Error),
    SMTP(lettre::transport::smtp::Error),
    Foreign(Box<dyn error::Error + Send + Sync>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Config(ref err) => write!(f, "Config Error: {}", err),
            Self::ConfigWrite(ref err) => write!(f, "Config Error: {}", err),
            Self::AptCache(ref err) => {
                write!(
                    f,
                    "APT Cache Error(s): {:?}",
                    err.what().split(';').map(|msg| {
                        if msg.starts_with("E:") {
                            format!("Error: {}", &msg[2..]);
                        } else if msg.starts_with("W:") {
                            format!("Warning: {}", &msg[2..]);
                        } else {
                            format!("{}", err);
                        }
                    })
                )
            }
            Self::Email(ref err) => write!(f, "Email Error: {}", err),
            Self::SMTP(ref err) => write!(f, "SMTP Error: {}", err),
            Self::Foreign(ref err) => write!(f, "Unknown error: {}", err),
        }
    }
}

impl From<lettre::error::Error> for Error {
    fn from(err: lettre::error::Error) -> Self {
        Self::Email(err)
    }
}

impl From<lettre::transport::smtp::Error> for Error {
    fn from(err: lettre::transport::smtp::Error) -> Self {
        Self::SMTP(err)
    }
}

impl From<config::ConfigError> for Error {
    fn from(err: config::ConfigError) -> Self {
        Self::Config(err)
    }
}
