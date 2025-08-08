pub mod renderer;
pub mod shortcuts;

// Re-export the main types for convenience
pub use renderer::{MenuActions, MenuRenderer};
pub use shortcuts::{ShortcutAction, ShortcutHandler};
