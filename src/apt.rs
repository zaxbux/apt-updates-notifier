use crate::error::{Error, Result};
use rust_apt::{
    cache::PackageSort,
    new_cache,
    raw::progress::AcquireProgress,
    util::{time_str, unit_str, NumSys},
};
use std::{
    fmt,
    sync::{Arc, Mutex},
};

/// acquire-item.h
#[repr(u32)]
#[derive(Clone, Debug, PartialEq)]
pub enum ItemState {
    /// The item is waiting to be downloaded.
    StatIdle,

    /// The item is currently being downloaded.
    StatFetching,

    /// The item has been successfully downloaded.
    StatDone,

    /// An error was encountered while downloading this item.
    StatError,

    /// The item was downloaded but its authenticity could not be verified.
    StatAuthError,

    /// The item was could not be downloaded because of a transient network error (e.g. network down)
    StatTransientNetworkError,
}

impl TryFrom<u32> for ItemState {
    type Error = ();

    fn try_from(value: u32) -> std::result::Result<Self, Self::Error> {
        match value {
            x if x == ItemState::StatIdle as u32 => Ok(ItemState::StatIdle),
            x if x == ItemState::StatFetching as u32 => Ok(ItemState::StatFetching),
            x if x == ItemState::StatDone as u32 => Ok(ItemState::StatDone),
            x if x == ItemState::StatError as u32 => Ok(ItemState::StatError),
            x if x == ItemState::StatAuthError as u32 => Ok(ItemState::StatAuthError),
            x if x == ItemState::StatTransientNetworkError as u32 => {
                Ok(ItemState::StatTransientNetworkError)
            }
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug)]
pub enum ProgressOutput {
    Hit(u32, String),
    Fetch(u32, String, u64),
    Stop(u64, u64, u64),
    Fail(u32, String, ItemState, String),
    Error(OutputError),
}

impl fmt::Display for ProgressOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hit(id, description) => write!(f, "Hit:{id} {description}"),
            Self::Fetch(id, description, file_size) => write!(
                f,
                "Get:{id} {description} [{}]",
                unit_str(*file_size, rust_apt::util::NumSys::Decimal)
            ),
            Self::Stop(fetched_bytes, elapsed_time, current_cps) => {
                if *fetched_bytes != 0 {
                    write!(
                        f,
                        "Fetched {} in {} ({}/s)",
                        unit_str(*fetched_bytes, NumSys::Decimal),
                        time_str(*elapsed_time),
                        unit_str(*current_cps, NumSys::Decimal)
                    )
                } else {
                    write!(f, "Nothing to fetch.")
                }
            }
            Self::Fail(id, description, status, _error_text) => {
                if *status == ItemState::StatIdle || *status == ItemState::StatDone {
                    return write!(f, "Ign:{id} {description}");
                } else {
                    return write!(f, "Err:{id} {description}");
                }
            }
            Self::Error(err) => match err {
                OutputError::Error(msg) => write!(f, "E: {}", &msg[2..]),
                OutputError::Warning(msg) => write!(f, "W: {}", &msg[2..]),
                OutputError::Notice(msg) => write!(f, "N: {}", &msg[2..]),
                OutputError::Debug(msg) => write!(f, "D: {}", &msg[2..]),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum OutputError {
    Error(String),
    Warning(String),
    Notice(String),
    Debug(String),
}

impl TryFrom<&str> for OutputError {
    type Error = String;

    fn try_from(value: &str) -> std::result::Result<Self, String> {
        if value.starts_with("E:") {
            Ok(Self::Error(value.to_string()))
        } else if value.starts_with("W:") {
            Ok(Self::Warning(value.to_string()))
        } else if value.starts_with("N:") {
            Ok(Self::Notice(value.to_string()))
        } else if value.starts_with("D:") {
            Ok(Self::Debug(value.to_string()))
        } else {
            Err(value.to_string())
        }
    }
}

pub fn update() -> Result<Vec<ProgressOutput>> {
    let cache = new_cache!().map_err(Error::AptCache)?;

    //let output: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let output: Arc<Mutex<Vec<ProgressOutput>>> = Arc::new(Mutex::new(Vec::new()));
    let mut progress: Box<dyn AcquireProgress> = Box::new(AptAcquireProgress::new(output.clone()));

    let result = cache.update(&mut progress); //.map_err(Error::AptCache);

    let mut output = output.lock().unwrap().clone();

    if let Err(err) = result {
        output.extend(
            err.what()
                .split(';')
                .map(|msg| ProgressOutput::Error(OutputError::try_from(msg).unwrap())),
        );
    }

    Ok(output)
}

pub fn list_upgradeable() -> Result<Vec<Upgradeable>> {
    let cache = new_cache!().map_err(Error::AptCache)?;
    let sort = PackageSort::default().upgradable().names();

    Ok(cache.packages(&sort).map(Into::<Upgradeable>::into).collect())
}

pub struct Upgradeable {
    pub name: String,
    pub installed: Option<String>,
    pub candidate: Option<String>,
    pub archive: String,
    pub arch: String,
}

impl From<rust_apt::package::Package<'_>> for Upgradeable {
    fn from(package: rust_apt::package::Package) -> Self {

        let inst = package.versions().next().unwrap();
        let package_files = inst.package_files();

        let archives: Vec<String> = package_files.filter_map(|p| p.archive().ok().map(|v| v.to_string()).or(Some("unknown".to_string()))).collect();
        let archive = archives.join(",");

        Self {
            name: package.name().to_string(),
            installed: package
                .installed()
                .map(|v| v.version().to_string()),
            candidate: package
                .candidate()
                .map(|v| v.version().to_string()),
            arch: package
            .candidate()
            .map(|v| v.arch().to_string()).unwrap_or_default(),
            archive,
        }
    }
}

#[derive(Debug)]
pub struct AptAcquireProgress {
    output: Arc<Mutex<Vec<ProgressOutput>>>,
}

impl AptAcquireProgress {
    /// Returns a new default progress instance.
    pub fn new(output: Arc<Mutex<Vec<ProgressOutput>>>) -> AptAcquireProgress {
        AptAcquireProgress { output }
    }
}

impl AcquireProgress for AptAcquireProgress {
    /// Pulse Interval set to 0 assumes the apt defaults.
    fn pulse_interval(&self) -> usize {
        0
    }

    /// Called when an item is confirmed to be up-to-date.
    fn hit(&mut self, id: u32, description: String) {
        self.output.lock().unwrap().push(ProgressOutput::Hit(id, description));
    }

    /// Called when an Item has started to download
    ///
    /// Prints out the short description and the expected size.
    fn fetch(&mut self, id: u32, description: String, file_size: u64) {
        self.output.lock().unwrap().push(ProgressOutput::Fetch(id, description, file_size));
    }

    /// Called when an item is successfully and completely fetched.
    fn done(&mut self) {}

    /// Called when progress has started.
    fn start(&mut self) {}

    /// Called when progress has finished.
    fn stop(
        &mut self,
        fetched_bytes: u64,
        elapsed_time: u64,
        current_cps: u64,
        pending_errors: bool,
    ) {
        if pending_errors {
            return;
        }

        self.output.lock().unwrap().push(ProgressOutput::Stop(
            fetched_bytes,
            elapsed_time,
            current_cps,
        ));
    }

    /// Called when an Item fails to download.
    ///
    /// Print out the ErrorText for the Item.
    fn fail(&mut self, id: u32, description: String, status: u32, error_text: String) {
        self.output.lock().unwrap().push(ProgressOutput::Fail(
            id,
            description,
            ItemState::try_from(status).unwrap(),
            error_text,
        ));
    }

    /// Called periodically to provide the overall progress information
    fn pulse(
        &mut self,
        _workers: Vec<rust_apt::raw::progress::Worker>,
        _percent: f32,
        _total_bytes: u64,
        _current_bytes: u64,
        _current_cps: u64,
    ) {
    }
}
