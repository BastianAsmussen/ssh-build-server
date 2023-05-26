use config::{Config, ConfigError};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub ssh: Ssh,
    pub compilation: Compilation,
    pub commands: Vec<Command>,
}

impl Settings {
    pub fn new(path: &str) -> Result<Self, ConfigError> {

        let default_config = Config::builder()
            .add_source(config::File::from_str(DEFAULT_SETTINGS, config::FileFormat::Toml))
            .build()?;

        // If the user did not supply a valid config path, we use the default config.
        match Config::builder()
            .add_source(config::File::with_name(path))
            .build()
        {
            Ok(config) => {
                // Merge the default config with the user-supplied config.
                let config = Config::builder()
                    .add_source(default_config)
                    .add_source(config)
                    .build()?;

                // Deserialize the config into a Settings instance.
                config.try_deserialize::<Self>()
            }
            Err(_) => {
                // Deserialize the default config into a Settings instance.
                default_config.try_deserialize::<Self>()
            }
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Ssh {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Compilation {
    pub local_project_root: String,
    pub remote_project_root: String,
    pub output_directory: String,
}

impl Compilation {
    /// Gets the remote output directory.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::util::settings::Settings;
    ///
    /// let settings = Settings::new("Settings.toml").unwrap();
    /// let remote_output_directory = settings.compilation.get_remote_output_directory();
    ///
    /// println!("Remote output directory: {}", remote_output_directory);
    /// ```
    pub fn get_remote_output_directory(&self) -> String {
        format!("{}/{}", self.remote_project_root, self.output_directory)
    }

    /// Gets the local output directory.
    ///
    /// # Example
    ///
    /// ```
    /// use crate::util::settings::Settings;
    ///
    /// let settings = Settings::new("Settings.toml").unwrap();
    /// let local_output_directory = settings.compilation.get_local_output_directory();
    ///
    /// println!("Local output directory: {}", local_output_directory);
    /// ```
    pub fn get_local_output_directory(&self) -> String {
        format!("{}/{}", self.local_project_root, self.output_directory)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Command {
    pub command: String,
    pub description: String,
    pub execute_after_compilation: bool,
}

/// The default settings profile.
pub const DEFAULT_SETTINGS: &str = r##"
[ssh]
host = "localhost"
port = 22
username = "root"
password = "root"

[compilation]
local_project_root = "/path/to/project" # The path to the project on your local machine from the root of the project.
remote_project_root = "~/remote/project" # The path to the project on the remote machine from the root of the project.
output_directory = "target/release" # The directory where the compiled binary is located relative to the project root.

[[commands]]
command = "cd ~/remote/project"
description = "Change directory to the project root."
execute_after_compilation = false

[[commands]]
command = "cargo build --release"
description = "Build the project."
execute_after_compilation = false
"##;