use std::path::PathBuf;

use crate::engine::{
    Error,
    models::runtime::{InteractiveMode, RunMode, Runtime},
    system::ui::Ui,
};

pub type AppResult<T> = Result<T, Error>;

#[derive(Debug)]
pub struct Context {
    runtime: Runtime,
}

impl Context {
    pub fn new(runtime: Runtime) -> Self {
        Self { runtime }
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn set_workdir(&mut self, path: PathBuf) {
        self.runtime.set_cwd(path);
    }

    pub fn set_run_mode(&mut self, str: &str) {
        self.runtime.set_run_mode(RunMode::parse(str));
    }

    pub fn set_interactive_mode(&mut self, ci: bool, non_interactive: bool) {
        self.runtime
            .set_interactive_mode(InteractiveMode::from_flags(ci, non_interactive));
    }

    pub fn is_interactive(&self) -> bool {
        self.runtime.is_interactive()
    }

    pub fn version(&self) -> &str {
        self.runtime.version()
    }

    pub fn cwd(&self) -> &PathBuf {
        self.runtime.cwd()
    }

    pub fn dry_run(&self) -> bool {
        self.runtime.options().dry_run()
    }

    pub fn auto_yes(&self) -> bool {
        self.runtime.options().auto_yes()
    }

    pub fn force(&self) -> bool {
        self.runtime.options().force()
    }

    pub fn ui_config(&self) -> scriba::Config {
        self.runtime.output_config()
    }

    pub fn ui(&self) -> Ui {
        Ui::cached_with_config(self.ui_config(), self.runtime.options().output_envelope())
    }
}
