use clap::ValueEnum;
use std::{
    cmp::Ordering,
    collections::HashMap,
    error::Error,
    fmt,
    fs::{self, File},
    io::{Read, Write},
    path::Path,
    process::exit,
    str::FromStr,
};

use serde::{Deserialize, Serialize};

pub mod rocket_models;
pub mod rocket_utils;
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
pub struct ConsumerDBSchema {
    pub url: String,
    pub lang: LANG,
    pub orm: ORM,
    pub root: String,
    pub schema_id: Option<String>,
    pub cache_schema_id: Option<String>,
    pub message_queue_schema_id: Option<String>,
    pub branch: Option<String>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ConsumerDBTables {
    pub names: Vec<String>,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct ConsumerDBConfig {
    pub schema: ConsumerDBSchema,
    pub tables: ConsumerDBTables,
}

pub fn write_consumer_db_config<P: AsRef<Path>>(path: P, config: &ConsumerDBConfig) -> () {
    let toml_string = toml::to_string(config).unwrap();
    let mut file = File::create(path).unwrap();
    file.write_all(toml_string.as_bytes()).unwrap();
}

pub fn read_consumer_db_config<P: AsRef<Path>>(
    path: P,
) -> Result<ConsumerDBConfig, Box<dyn std::error::Error>> {
    // Open the file
    let mut file = File::open(path).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    // Deserialize the TOML contents into the DBConfig struct
    match toml::from_str(&contents) {
        Ok(config) => Ok(config),
        Err(err) => Err(Box::new(err)),
    }
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
    Shell,
}

impl fmt::Display for LANG {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LANG::Rust => write!(f, "Rust"),
            LANG::TS => write!(f, "TS"),
            LANG::Python => write!(f, "Python"),
            LANG::Shell => write!(f, "Shell"),
        }
    }
}

impl LANG {
    pub fn all() -> Vec<LANG> {
        vec![LANG::Rust, LANG::TS, LANG::Python, LANG::Shell]
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct ServiceConfig {
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Link {
    pub internal: bool,
    pub label: String,
    pub icon: String,
    pub link: String,
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[internal: {}, label: {}, icon: {}]",
            self.internal, self.label, self.icon
        )
    }
}

impl PartialEq for Link {
    fn eq(&self, other: &Self) -> bool {
        self.internal == other.internal
            && self.label == other.label
            && self.icon == other.icon
            && self.link == other.link
    }
}

#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct PackageMetadata {
    pub lang: LANG,
    pub package_type: String,
    #[serde(default = "default_links")]
    pub links: Vec<Link>,
}

fn default_links() -> Vec<Link> {
    vec![]
}

#[derive(Debug, Clone)]
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
#[derive(Debug, Serialize, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
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
    #[serde(default = "default_take_snapshots")]
    pub take_snapshots: bool,
}

fn default_take_snapshots() -> bool {
    false
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReleaserConfig {
    pub settings: ReleaserSettings,
    pub version: Version,
    #[serde(default = "default_references")]
    pub references: Vec<Reference>,
}

fn default_references() -> Vec<Reference> {
    vec![]
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

pub fn write_releaser_config_file(
    file_path: &str,
    config: &ReleaserConfig,
) -> Result<(), Box<dyn Error>> {
    let toml_str = toml::to_string(config)?;
    fs::write(file_path, toml_str)?;
    Ok(())
}

pub fn read_service_config_file<P: AsRef<Path>>(path: P) -> Result<ServiceConfig, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let config: ServiceConfig = toml::from_str(&content)?;
    Ok(config)
}

pub fn read_package_metadata_file<P: AsRef<Path>>(
    path: P,
) -> Result<PackageMetadata, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let config: PackageMetadata = toml::from_str(&content)?;
    Ok(config)
}

pub fn write_service_config_file<P: AsRef<Path>>(
    path: P,
    config: &ServiceConfig,
) -> Result<(), Box<dyn Error>> {
    let content = toml::to_string(config)?;
    fs::write(path, content)?;
    Ok(())
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct GingerDBConfig {
    pub branch: String,
    pub organization_id: String,
    pub database: Vec<DatabaseConfig>, // Unified all db types in one vector
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct DatabaseConfig {
    pub db_type: DbType, // Use DbType enum
    pub description: String,
    pub enable: bool,
    pub id: Option<String>,
    pub name: String,
    pub port: String,
    pub studio_port: Option<String>,
    #[serde(default = "default_links")]
    pub links: Vec<Link>,
}

impl fmt::Display for DatabaseConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")] // This will map the enum to/from lowercase strings
pub enum DbType {
    Rdbms,
    DocumentDb,
    Cache,
    MessageQueue,
}

impl fmt::Display for DbType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let db_type_str = match self {
            DbType::Rdbms => "rdbms",
            DbType::DocumentDb => "documentdb",
            DbType::Cache => "cache",
            DbType::MessageQueue => "messagequeue",
        };
        write!(f, "{}", db_type_str)
    }
}

impl FromStr for DbType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "rdbms" => Ok(DbType::Rdbms),
            "documentdb" => Ok(DbType::DocumentDb),
            "cache" => Ok(DbType::Cache),
            "messagequeue" => Ok(DbType::MessageQueue),
            _ => Err(format!("'{}' is not a valid DbType", s)),
        }
    }
}

pub fn read_db_config(file_path: &str) -> Result<GingerDBConfig, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(file_path)?;
    let config: GingerDBConfig = toml::from_str(&contents)?;
    Ok(config)
}

pub fn write_db_config(
    file_path: &str,
    config: &GingerDBConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let toml_string = toml::to_string(config)?;
    let mut file = fs::File::create(file_path)?;
    file.write_all(toml_string.as_bytes())?;
    Ok(())
}

#[derive(ValueEnum, Clone, PartialEq)]
pub enum Environment {
    Dev,
    Stage,
    Prod,
    ProdK8,
    StageK8,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Environment::Dev => write!(f, "dev"),
            Environment::Stage => write!(f, "stage"),
            Environment::Prod => write!(f, "prod"),
            Environment::ProdK8 => write!(f, "prod_k8"),
            Environment::StageK8 => write!(f, "stage_k8"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ISCClaims {
    pub sub: String,
    pub exp: usize,
    pub org_id: String,
    pub scopes: Vec<String>,
}
