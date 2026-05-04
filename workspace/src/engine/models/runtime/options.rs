#[derive(Debug, Clone)]
pub struct RuntimeOptions {
    dry_run: bool,
    auto_yes: bool,
    force: bool,
    output_envelope: scriba::EnvelopeMode,
    output_format: scriba::Format,
    output_color: scriba::ColorMode,
    log_level: scriba::Level,
}

impl Default for RuntimeOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeOptions {
    pub fn new() -> Self {
        Self {
            dry_run: false,
            auto_yes: false,
            force: false,
            output_envelope: scriba::EnvelopeMode::None,
            output_format: scriba::Format::Text,
            output_color: scriba::ColorMode::Auto,
            log_level: scriba::Level::Normal,
        }
    }

    pub fn dry_run(&self) -> bool {
        self.dry_run
    }

    pub fn set_dry_run(&mut self, dry_run: bool) -> &mut Self {
        self.dry_run = dry_run;
        self
    }

    pub fn auto_yes(&self) -> bool {
        self.auto_yes
    }

    pub fn set_auto_yes(&mut self, auto_yes: bool) -> &mut Self {
        self.auto_yes = auto_yes;
        self
    }

    pub fn force(&self) -> bool {
        self.force
    }

    pub fn set_force(&mut self, force: bool) -> &mut Self {
        self.force = force;
        self
    }

    pub fn output_envelope(&self) -> scriba::EnvelopeMode {
        self.output_envelope
    }

    pub fn set_output_envelope(&mut self, output_envelope: scriba::EnvelopeMode) -> &mut Self {
        self.output_envelope = output_envelope;
        self
    }

    pub fn output_format(&self) -> scriba::Format {
        self.output_format
    }

    pub fn set_output_format(&mut self, output_format: scriba::Format) -> &mut Self {
        self.output_format = output_format;
        self
    }

    pub fn output_color(&self) -> scriba::ColorMode {
        self.output_color
    }

    pub fn set_output_color(&mut self, output_color: scriba::ColorMode) -> &mut Self {
        self.output_color = output_color;
        self
    }

    pub fn log_level(&self) -> scriba::Level {
        self.log_level
    }

    pub fn set_log_level(&mut self, log_level: scriba::Level) -> &mut Self {
        self.log_level = log_level;
        self
    }
}
