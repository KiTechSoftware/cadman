/// Terminal utilities for adaptive output formatting
use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct TerminalSize {
    pub width: u16,
    pub height: u16,
}

impl TerminalSize {
    /// Get current terminal size
    pub fn current() -> Self {
        termsize::get()
            .map(|size| Self {
                width: size.cols,
                height: size.rows,
            })
            .unwrap_or_default()
    }

    /// Check if terminal width is considered narrow (<= 80 chars)
    pub fn is_narrow(self) -> bool {
        self.width <= 80
    }

    /// Check if terminal width is considered standard (81-120 chars)
    pub fn is_standard(self) -> bool {
        self.width > 80 && self.width <= 120
    }

    /// Check if terminal width is considered wide (> 120 chars)
    pub fn is_wide(self) -> bool {
        self.width > 120
    }

    /// Get appropriate scriba table layout for terminal size
    pub fn table_layout(self) -> scriba::TableLayout {
        match self.width {
            0..=80 => scriba::TableLayout::Stacked,
            81..=120 => scriba::TableLayout::Compact,
            _ => scriba::TableLayout::Full,
        }
    }

    /// Get safe column width accounting for borders and padding
    pub fn safe_column_width(self, num_columns: usize) -> u16 {
        if num_columns == 0 {
            return self.width;
        }

        let available = self.width.saturating_sub(2 + (num_columns as u16 * 2)); // 2 for borders, 2 per column for spacing
        (available / num_columns as u16).max(10)
    }
}

impl Default for TerminalSize {
    fn default() -> Self {
        Self {
            width: 120,
            height: 24,
        }
    }
}

impl fmt::Display for TerminalSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_size_classification() {
        assert!(
            TerminalSize {
                width: 80,
                height: 24
            }
            .is_narrow()
        );
        assert!(
            TerminalSize {
                width: 100,
                height: 24
            }
            .is_standard()
        );
        assert!(
            TerminalSize {
                width: 160,
                height: 24
            }
            .is_wide()
        );
    }

    #[test]
    fn test_table_layout() {
        assert_eq!(
            TerminalSize {
                width: 80,
                height: 24
            }
            .table_layout(),
            scriba::TableLayout::Stacked
        );
        assert_eq!(
            TerminalSize {
                width: 100,
                height: 24
            }
            .table_layout(),
            scriba::TableLayout::Compact
        );
        assert_eq!(
            TerminalSize {
                width: 160,
                height: 24
            }
            .table_layout(),
            scriba::TableLayout::Full
        );
    }
}
