pub mod assertions;
pub mod build;
pub mod check;
pub mod error;
pub mod eval;
pub mod flake;
pub mod host;
pub mod options;
pub mod process;
pub mod value;

pub use assertions::{failed_assertions, parse_assertions, Assertion};
pub use build::{build as nix_build, build_dry_run, BuildResult};
pub use check::{flake_check, flake_check_strict, FlakeCheckResult};
pub use error::{NixError, NixResult};
pub use eval::{eval_attr, eval_attr_with_apply, eval_expr, eval_json};
pub use flake::{discover_external_modules, discover_inputs, ExternalModule, FlakeInput};
pub use host::{
    eval_home_manager, eval_home_manager_option, eval_host, eval_host_option, list_home_configs,
    list_hosts, HostEvalResult,
};
pub use options::{
    extract_option, extract_option_value, parse_nix_type_string, parse_option_schema,
};
pub use process::{NixProcess, NixProcessConfig, ProcessOutput};
pub use value::{decode_json, infer_nix_type, json_to_nix_value, value_to_json};

use std::path::{Path, PathBuf};

use avalanche_model::Scope;

pub struct NixService {
    repo_path: PathBuf,
    process: NixProcess,
}

impl NixService {
    pub fn new(repo_path: impl Into<String>) -> Self {
        Self {
            repo_path: PathBuf::from(repo_path.into()),
            process: NixProcess::default(),
        }
    }

    pub fn with_config(repo_path: impl Into<String>, config: NixProcessConfig) -> Self {
        Self {
            repo_path: PathBuf::from(repo_path.into()),
            process: NixProcess::new(config),
        }
    }

    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }

    pub fn process(&self) -> &NixProcess {
        &self.process
    }

    pub fn eval(&self, expr: &str) -> NixResult<String> {
        eval_expr(&self.process, &self.repo_path, expr)
    }

    pub fn eval_json(&self, expr: &str) -> NixResult<serde_json::Value> {
        eval_json(&self.process, &self.repo_path, expr)
    }

    pub fn flake_check(&self) -> NixResult<FlakeCheckResult> {
        flake_check(&self.process, &self.repo_path)
    }

    pub fn build(&self, attribute: &str) -> NixResult<BuildResult> {
        nix_build(&self.process, &self.repo_path, attribute)
    }

    pub fn build_dry_run(&self, attribute: &str) -> NixResult<BuildResult> {
        build_dry_run(&self.process, &self.repo_path, attribute)
    }

    pub fn list_hosts(&self) -> NixResult<Vec<String>> {
        list_hosts(&self.process, &self.repo_path)
    }

    pub fn eval_host(&self, host: &str) -> NixResult<HostEvalResult> {
        eval_host(&self.process, &self.repo_path, host)
    }

    pub fn eval_host_option(&self, host: &str, option_path: &str) -> NixResult<serde_json::Value> {
        eval_host_option(&self.process, &self.repo_path, host, option_path)
    }

    pub fn eval_home_manager(
        &self,
        host: &str,
        user: &str,
    ) -> NixResult<serde_json::Value> {
        eval_home_manager(&self.process, &self.repo_path, host, user)
    }

    pub fn eval_home_manager_option(
        &self,
        host: &str,
        user: &str,
        option_path: &str,
    ) -> NixResult<serde_json::Value> {
        eval_home_manager_option(&self.process, &self.repo_path, host, user, option_path)
    }

    pub fn extract_option(
        &self,
        option_path: &str,
        scope: Scope,
        host: &str,
    ) -> NixResult<avalanche_model::OptionSchema> {
        extract_option(&self.process, &self.repo_path, option_path, scope, host)
    }

    pub fn extract_option_value(
        &self,
        option_path: &str,
        scope: Scope,
        host: &str,
    ) -> NixResult<serde_json::Value> {
        extract_option_value(&self.process, &self.repo_path, option_path, scope, host)
    }

    pub fn get_assertions(&self, host: &str) -> NixResult<Vec<Assertion>> {
        assertions::get_assertions(&self.process, &self.repo_path, host)
    }

    pub fn discover_inputs(&self) -> NixResult<Vec<FlakeInput>> {
        discover_inputs(&self.process, &self.repo_path)
    }

    pub fn discover_external_modules(&self) -> NixResult<Vec<ExternalModule>> {
        discover_external_modules(&self.process, &self.repo_path)
    }
}
