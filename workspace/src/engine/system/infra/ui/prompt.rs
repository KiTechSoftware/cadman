use super::{map_scriba_error_to_cadman_error, UiResult};

impl super::Ui {
    fn text(&self, message: &str, default: Option<&str>, help: Option<&str>) -> UiResult<String> {
        scriba::prompt::text(&self.config, message, default, help, &self.prompt_theme)
            .map_err(map_scriba_error_to_cadman_error)
    }

    fn confirm(&self, message: &str, default: bool) -> UiResult<bool> {
        scriba::prompt::confirm(&self.config, message, default, &self.prompt_theme)
            .map_err(map_scriba_error_to_cadman_error)
    }

    fn select(&self, request: &scriba::prompt::SelectRequest) -> UiResult<String> {
        scriba::prompt::select(&self.config, request, &self.prompt_theme)
            .map_err(map_scriba_error_to_cadman_error)
    }

    fn multiselect(&self, request: &scriba::prompt::MultiSelectRequest) -> UiResult<Vec<String>> {
        scriba::prompt::multiselect(&self.config, request, &self.prompt_theme)
            .map_err(map_scriba_error_to_cadman_error)
    }
}
