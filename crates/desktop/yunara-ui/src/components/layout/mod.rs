/// Layout components for the application structure.
///
/// Contains the main structural components:
/// - `AppShell`: Main application frame combining all layout elements
/// - `Sidebar`: Left navigation and playlist listing
/// - `ContentArea`: Main content container
/// - `QueuePanel`: Right-side playback queue

mod app_shell;
mod content_area;
mod queue_panel;
mod sidebar;

pub use app_shell::AppShell;
pub use content_area::ContentArea;
pub use queue_panel::{QueuePanel, QueueTab};
pub use sidebar::Sidebar;
