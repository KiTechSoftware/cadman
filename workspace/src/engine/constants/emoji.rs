//! Emoji constants for UI messages.
//! Used to enhance readability and provide visual feedback in terminal output.
use veltrix::unicode::emojis;

/// Info/neutral status indicator
pub const INFO: &str = emojis::EMOJI_INFORMATION;

/// Success/positive status indicator
pub const SUCCESS: &str = emojis::EMOJI_CHECK_MARK_BUTTON;

/// Failure/negative status indicator
pub const ERROR: &str = emojis::EMOJI_CROSS_MARK_BUTTON;

/// File/folder indicator
pub const FILES: &str = emojis::EMOJI_FILE_FOLDER;

/// Search/preview/inspection indicator
pub const INSPECT: &str = emojis::EMOJI_MAGNIFYING_GLASS_TILTED_LEFT;

/// Changes indicator
pub const CHANGES: &str = emojis::EMOJI_MEMO;

/// Stage/add indicator
pub const STAGE: &str = emojis::EMOJI_PACKAGE;

/// Unsustage/remove indicator
pub const UNSTAGE: &str = emojis::EMOJI_LEFT_ARROW_CURVING_RIGHT;

/// Dry run/preview indicator
pub const DRY_RUN: &str = emojis::EMOJI_EYES;

/// Construction/in-progress indicator
pub const IN_PROGRESS: &str = emojis::EMOJI_HAMMER;

/// Preview/construction work indicator
pub const PREVIEW: &str = emojis::EMOJI_CONSTRUCTION;

/// Complete/finished indicator
pub const COMPLETE: &str = emojis::EMOJI_PARTY_POPPER;

/// Cleanup/remove indicator
pub const CLEANUP: &str = emojis::EMOJI_BROOM;

/// Warning indicator
pub const WARN: &str = emojis::EMOJI_WARNING;

/// Arrow/direction indicator
pub const ARROW: &str = emojis::EMOJI_RIGHT_ARROW;

/// Checkmark for list items
pub const CHECK: &str = emojis::EMOJI_CHECK_MARK;

/// Cross for list items
pub const CROSS: &str = emojis::EMOJI_CROSS_MARK;
