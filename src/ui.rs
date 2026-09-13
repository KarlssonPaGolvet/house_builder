//! Core UI module declaring submodules and re-exporting UI components,
//! helpers, and systems for seamless use across the application.

/// Common UI marker components and types.
pub mod components;

/// Helper functions for UI widget creation, parsing, and layout formatting.
pub mod helpers;

/// In-game HUD elements, debug readouts, and Egui control windows.
pub mod hud;

/// Main menu navigation screens, saved house lists, and new world creation.
pub mod main_menu;

/// In-game pause menu and controls guide overlays.
pub mod pause_menu;

// Re-export all submodules to maintain flat module access throughout the codebase.
pub use components::*;
pub use helpers::*;
pub use hud::*;
pub use main_menu::*;
pub use pause_menu::*;