// --- std ---
use std::{
	fs::{create_dir_all, File},
	io::{Read, Write},
};
// --- crates.io ---
use app_dirs2::{get_app_root, AppDataType};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
// --- subalfred ---
use crate::{Result, APP_INFO};

lazy_static! {
	pub static ref CONFIG: Config = Config::load_config();
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
	pub github_oauth_token: String,
	pub substrate_project: SubstrateProject,
}
impl Config {
	pub fn load_config() -> Self {
		let app_root_path = get_app_root(AppDataType::UserConfig, &APP_INFO).unwrap();
		let app_config_path = app_root_path.join("config");
		let file = if app_config_path.is_file() {
			File::with_options()
				.create(false)
				.read(true)
				.write(true)
				.append(false)
				.open(&app_config_path)
				.unwrap()
		} else {
			if !app_root_path.is_dir() {
				create_dir_all(&app_root_path).unwrap();
			}

			let mut file = File::with_options();

			file.create_new(true).read(true).write(true).append(false);

			#[cfg(target_family = "unix")]
			{
				// --- std ---
				use std::os::unix::fs::OpenOptionsExt;

				file.mode(0o600);
			}

			file.open(&app_config_path).unwrap()
		};

		if let Ok(config) = Config::from_reader(&file) {
			config
		} else {
			let config = Config::default();

			config.to_writer(&file).unwrap();

			config
		}
	}

	pub fn from_reader(r: impl Read) -> Result<Self> {
		serde_yaml::from_reader(r).map_err(Into::into)
	}

	pub fn to_writer(&self, w: impl Write) -> Result<()> {
		// TODO
		// serde_yaml::to_writer(w, self).map_err(Into::into)

		const TEMPLATE: &'static [u8] =
br#"
# Your github OAuth token
#
# You can use OAuth tokens to interact with GitHub via automated scripts.
#   https://docs.github.com/en/free-pro-team@latest/github/extending-github/git-automation-with-oauth-tokens
# Get your personal access token.
#   https://docs.github.com/en/free-pro-team@latest/github/authenticating-to-github/creating-a-personal-access-token
#
# Use for fetching substrate updates
# Or create/upgrade issue for you substrate project (require write access if you want to use these features)
github-oauth-token: "oauth-token"

# Your substrate project information
substrate-project:
  # https://github.com/{owner}/{repo}
  owner: "owner"
  repo: "repo"
  issue-repo: "issue-repo"
  local-full-path: "/path/to/project"
  runtimes: [
    runtime:
      runtime-relative-paths: "/path/to/runtime/src/lib.rs"
      node-rpc-address: "http://127.0.0.1:9933"
  ]
"#;

		let mut w = w;

		w.write_all(TEMPLATE)?;
		w.flush().map_err(Into::into)
	}
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SubstrateProject {
	pub owner: String,
	pub repo: String,
	pub issue_repo: String,
	pub local_full_path: String,
	pub runtimes: Vec<Runtime>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Runtime {
	pub runtime_relative_path: String,
	pub node_rpc_address: String,
}
