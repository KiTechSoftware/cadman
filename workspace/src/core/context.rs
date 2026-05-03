use crate::engine::{Error, models::runtime::Runtime, system::ui::Ui};

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

    pub fn ui(&self) -> Ui {
        Ui::new()
    }
}
