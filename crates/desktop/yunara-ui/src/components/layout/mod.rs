// Copyright 2025 Crrow
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// Layout components for the application structure.
///
/// Contains the main structural components:
/// - `Header`: Top navigation bar with logo and search
/// - `Sidebar`: Left navigation and playlist listing
/// - `ContentArea`: Main content container
/// - `QueuePanel`: Right-side playback queue
mod content_area;
mod header;
mod queue_panel;
mod sidebar;

pub use content_area::ContentArea;
pub use header::Header;
pub use queue_panel::{QueuePanel, QueueTab};
pub use sidebar::Sidebar;
