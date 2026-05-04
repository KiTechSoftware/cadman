use std::cell::RefCell;
use std::io::{self, Write};
mod logger;
mod prompt;

use crate::engine::{Error, ErrorCode};
use scriba::{Config, Format, Logger, Meta, Output, envelope, output::render};
pub type UiResult<T> = Result<T, Error>;

// Thread-local cache for the current Ui instance
thread_local! {
    static UI_CACHE: RefCell<Option<Ui>> = const { RefCell::new(None) };
}

#[derive(Debug, Clone)]
pub struct Ui {
    config: Config,
    envelope: envelope::EnvelopeConfig,
    prompt_theme: scriba::prompt::PromptTheme,
}

impl Ui {
    pub fn new() -> Self {
        Self::with_config(Config::default())
    }

    pub fn with_config(config: Config) -> Self {
        Self {
            config,
            envelope: envelope::EnvelopeConfig::default(),
            prompt_theme: scriba::prompt::PromptTheme::default(),
        }
    }

    /// Returns a cached Ui instance configured with the given config and envelope mode.
    /// The cache is per-thread and keyed by config. If the same config is requested,
    /// the cached instance is returned, avoiding repeated creation.
    pub fn cached_with_config(config: Config, envelope_mode: envelope::EnvelopeMode) -> Self {
        UI_CACHE.with(|cache| {
            let mut cached = cache.borrow_mut();
            match cached.as_ref() {
                Some(ui) if ui.config == config && ui.envelope.mode == envelope_mode => ui.clone(),
                _ => {
                    let ui = Self::with_config(config).with_envelope_mode(envelope_mode);
                    *cached = Some(ui.clone());
                    ui
                }
            }
        })
    }

    /// Get reference to the current configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn use_color(&self) -> bool {
        if self.config.color == scriba::ColorMode::Always
            || (self.config.color == scriba::ColorMode::Auto && self.config.interactive)
        {
            return true;
        }
        false
    }

    /// Get reference to the current envelope configuration.
    pub fn envelope(&self) -> &envelope::EnvelopeConfig {
        &self.envelope
    }

    pub fn with_envelope(mut self, config: envelope::EnvelopeConfig) -> Self {
        self.envelope = config;
        self
    }

    pub fn with_envelope_mode(mut self, mode: envelope::EnvelopeMode) -> Self {
        self.envelope.mode = mode;
        self
    }

    pub fn with_envelope_layout(mut self, layout: envelope::EnvelopeLayout) -> Self {
        self.envelope.layout = layout;
        self
    }

    pub fn with_envelope_fields(mut self, fields: envelope::EnvelopeFields) -> Self {
        self.envelope.fields = fields;
        self
    }

    pub fn with_prompt_theme(mut self, theme: scriba::prompt::PromptTheme) -> Self {
        self.prompt_theme = theme;
        self
    }

    pub fn prompt_theme(&self) -> &scriba::prompt::PromptTheme {
        &self.prompt_theme
    }

    pub fn with_format(mut self, format: Format) -> Self {
        self.config.format = format;
        self
    }

    pub fn interactive(mut self, value: bool) -> Self {
        self.config.interactive = value;
        self
    }

    pub fn auto_yes(mut self, value: bool) -> Self {
        self.config.auto_yes = value;
        self
    }

    pub fn logger(&self) -> Logger<'_> {
        Logger::new(&self.config)
    }

    pub fn new_output_content(&self) -> Output {
        Output::new()
    }

    pub fn new_output_meta(&self) -> Meta {
        Meta::default()
    }

    pub fn render(&self, output: &Output) -> UiResult<String> {
        render::render_output(self.config.format, output).map_err(map_scriba_error_to_cadman_error)
    }

    pub fn print(&self, output: &Output) -> UiResult<()> {
        self.print_with_meta(output, None, true)
    }

    pub fn print_with_meta(&self, output: &Output, meta: Option<&Meta>, ok: bool) -> UiResult<()> {
        let text = if self.envelope.mode.is_json() {
            let content = render::render_output_value(self.config.format, output)
                .map_err(map_scriba_error_to_cadman_error)?;
            let wrapped = envelope::wrap(
                &self.envelope,
                self.config.format.as_str(),
                content,
                meta,
                ok,
            );
            serde_json::to_string_pretty(&wrapped)?
        } else {
            self.render(output)?
        };
        let mut stdout = io::stdout();
        stdout.write_all(text.as_bytes())?;
        stdout.flush()?;
        Ok(())
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self::new()
    }
}

fn map_scriba_error_to_cadman_error(err: scriba::Error) -> Error {
    // we should convert the scriba error into a cadman error, but for now we just wrap it
    ErrorCode::IoFailure
        .error()
        .with_context("error", err.to_string())
}
