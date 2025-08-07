pub mod renderer;
pub mod shortcuts;

// Re-export the main types for convenience
pub use renderer::{MenuRenderer, MenuActions};
pub use shortcuts::{ShortcutHandler, ShortcutAction};
