use secrecy::ExposeSecret;
use secrecy::Secret;
use serde_aux::prelude::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgSslMode;
use sqlx::ConnectOptions;

#[derive(serde::Deserialize, Clone, Debug)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub foreign_database: DatabaseSettings,
    pub content: ContentSettings,
    pub metrics: MetricsSettings,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,

    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

impl DatabaseSettings {
    #[must_use]
    pub fn with_db(&self) -> PgConnectOptions {
        let mut options = self.without_db().database(&self.database_name);

        options.log_statements(tracing::log::LevelFilter::Trace);
        options
    }

    #[must_use]
    pub fn without_db(&self) -> PgConnectOptions {
        let ssl_mode = if self.require_ssl {
            PgSslMode::Require
        } else {
            PgSslMode::Prefer
        };

        PgConnectOptions::new()
            .host(&self.host)
            .username(&self.username)
            .password(self.password.expose_secret())
            .port(self.port)
            .ssl_mode(ssl_mode)
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct ContentSettings {
    pub deregistered_nodes: Vec<String>,

    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub signature_spid: u16,

    pub delegate_priv_key: Secret<String>,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct MetricsSettings {
    pub foundation_nodes: Vec<u16>,
    pub push_gateway: String,
    pub slack_url: String,
}

pub enum Environment {
    Stage,
    Production,
}

impl Environment {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Stage => "stage",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "stage" => Ok(Self::Stage),
            "production" => Ok(Self::Production),
            other => Err(format!(
                "{other} is not a supported environment. Use either `stage` or `production`",
            )),
        }
    }
}

pub fn read() -> Result<Settings, config::ConfigError> {
    let mut builder = config::Config::builder();

    let base_path = std::env::current_dir().expect("Failed to determine the current directory");

    let configuration_directory = base_path.join("configuration");

    builder =
        builder.add_source(config::File::from(configuration_directory.join("base")).required(true));

    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "stage".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    builder = builder.add_source(
        config::File::from(configuration_directory.join(environment.as_str())).required(true),
    );

    // Add in settings from environment variables (with a prefix of APP and '__' as separator)
    // E.g. `APP_APPLICATION__PORT=5001 would set `Settings.application.port`

    builder = builder.add_source(
        config::Environment::with_prefix("APP")
            .prefix_separator("_")
            .separator("__"),
    );

    let settings = builder.build()?;
    settings.try_deserialize::<Settings>()
}
