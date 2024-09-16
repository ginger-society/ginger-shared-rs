use clap::ValueEnum;
use std::{cmp::Ordering, collections::HashMap, error::Error, fmt, fs, path::Path, process::exit};

use serde::{Deserialize, Serialize};

pub mod utils;

#[derive(Debug, Serialize, Deserialize)]
pub enum ORM {
    TypeORM,
    SQLAlchemy,
    DjangoORM,
    Diesel,
}

impl fmt::Display for ORM {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ORM::TypeORM => write!(f, "TypeORM"),
            ORM::SQLAlchemy => write!(f, "SQLAlchemy"),
            ORM::DjangoORM => write!(f, "DjangoORM"),
            ORM::Diesel => write!(f, "Diesel"),
        }
    }
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DBSchema {
    pub url: String,
    pub lang: LANG,
    pub orm: ORM,
    pub root: String,
    pub schema_id: Option<String>,
    pub cache_schema_id: Option<String>,
    pub branch: Option<String>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DBTables {
    pub names: Vec<String>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DBConfig {
    pub schema: DBSchema,
    pub tables: DBTables,
}

#[derive(Debug, Clone)]
pub struct Service {
    pub schema_url: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, ValueEnum)]
pub enum LANG {
    Rust,
    TS,
    Python,
}

impl fmt::Display for LANG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LANG::Rust => write!(f, "rust"),
            LANG::TS => write!(f, "typescript"),
            LANG::Python => write!(f, "python"),
        }
    }
}

impl LANG {
    pub fn all() -> Vec<LANG> {
        vec![LANG::Rust, LANG::TS, LANG::Python]
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct Config {
    pub services: Option<HashMap<String, HashMap<String, String>>>,
    pub portals_refs: Option<HashMap<String, HashMap<String, String>>>,
    pub lang: LANG,
    pub organization_id: String,
    pub dir: Option<String>, // in case the project does not need any service integration
    pub portal_refs_file: Option<String>,
    pub spec_url: Option<String>,
    pub urls: Option<HashMap<String, String>>,
    pub override_name: Option<String>,
    pub service_type: Option<String>,
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct PackageMetadata {
    pub lang: LANG,
    pub package_type: String,
}

#[derive(Debug)]
pub enum FileType {
    Py,
    Toml,
    Json,
    Unknown,
}

impl FileType {
    pub fn from_extension(ext: Option<&str>) -> FileType {
        match ext {
            Some("py") => FileType::Py,
            Some("toml") => FileType::Toml,
            Some("json") => FileType::Json,
            _ => FileType::Unknown,
        }
    }
}

impl fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileType::Py => write!(f, "Py"),
            FileType::Toml => write!(f, "Toml"),
            FileType::Json => write!(f, "Json"),
            FileType::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum Channel {
    Final,
    Nightly, // Also known as Dev branch
    Alpha,
    Beta,
}
impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Channel::Nightly => write!(f, "nightly"),
            Channel::Final => write!(f, "final"),
            Channel::Alpha => write!(f, "alpha"),
            Channel::Beta => write!(f, "beta"),
        }
    }
}

impl From<&str> for Channel {
    fn from(channel: &str) -> Self {
        match channel {
            "nightly" => Channel::Nightly,
            "alpha" => Channel::Alpha,
            "beta" => Channel::Beta,
            "final" => Channel::Final,
            c => {
                println!("Unable to recognize the channel {:?}", c);
                exit(1)
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, Eq)]
pub struct Version {
    pub channel: Channel,
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub revision: u32,
}

impl Version {
    pub fn formatted(&self) -> String {
        match &self.channel {
            Channel::Final => {
                format!("{}.{}.{}", self.major, self.minor, self.patch)
            }
            _ => {
                format!(
                    "{}.{}.{}-{}.{}",
                    self.major, self.minor, self.patch, self.channel, self.revision
                )
            }
        }
    }
    pub fn tuple(&self) -> String {
        format!(
            "({}, {}, {}, \"{}\", {})",
            self.major, self.minor, self.patch, self.channel, self.revision
        )
    }

    pub fn from_str(version: &str) -> Self {
        let parts: Vec<&str> = version.split(|c| c == '.' || c == '-').collect();
        let major = parts[0].parse().unwrap_or(0);
        let minor = parts[1].parse().unwrap_or(0);
        let patch = parts[2].parse().unwrap_or(0);
        let (channel, revision) = if parts.len() > 3 {
            (Channel::from(parts[3]), parts[4].parse().unwrap_or(0))
        } else {
            (Channel::Final, 0)
        };

        Version {
            major,
            minor,
            patch,
            channel,
            revision,
        }
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then(self.minor.cmp(&other.minor))
            .then(self.patch.cmp(&other.patch))
            .then(self.channel.cmp(&other.channel))
            .then(self.revision.cmp(&other.revision))
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch == other.patch
            && self.channel == other.channel
            && self.revision == other.revision
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub enum OutputType {
    String,
    Tuple,
}

impl fmt::Display for OutputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputType::String => write!(f, "String"),
            OutputType::Tuple => write!(f, "Tuple"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Reference {
    pub file_name: String,
    #[serde(default = "default_output_type")] // Use a default value function
    pub output_type: OutputType, // `type` is a reserved keyword in Rust
    pub variable: String,
    #[serde(skip, default = "default_file_type")] // This field is not in the TOML file
    pub file_type: FileType,
}

fn default_file_type() -> FileType {
    FileType::Unknown
}

fn default_output_type() -> OutputType {
    OutputType::String // Default value is "string"
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReleaserSettings {
    pub git_url_prefix: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReleaserConfig {
    pub settings: ReleaserSettings,
    pub version: Version,
}

pub fn read_releaser_config_file<P: AsRef<Path>>(
    file_path: P,
) -> Result<ReleaserConfig, Box<dyn std::error::Error>> {
    // Read the file content into a string
    let contents = fs::read_to_string(file_path)?;

    // Parse the TOML string into the Settings struct
    let settings: ReleaserConfig = toml::de::from_str(&contents)?;

    Ok(settings)
}

pub fn read_config_file<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config)
}

pub fn read_package_metadata_file<P: AsRef<Path>>(
    path: P,
) -> Result<PackageMetadata, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let config: PackageMetadata = toml::from_str(&content)?;
    Ok(config)
}

pub fn write_config_file<P: AsRef<Path>>(path: P, config: &Config) -> Result<(), Box<dyn Error>> {
    let content = toml::to_string(config)?;
    fs::write(path, content)?;
    Ok(())
}
