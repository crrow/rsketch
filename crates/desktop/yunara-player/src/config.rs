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

use yunara_store::DatabaseConfig;

/// Application configuration
#[derive(Debug, Clone, Default, bon::Builder)]
pub struct AppConfig {
    /// Database configuration
    #[builder(getter)]
    pub database: DatabaseConfig,
    /// Application-level configuration
    #[builder(getter)]
    pub app:      ApplicationConfig,
}

/// Application-level configuration
#[derive(Debug, Clone, Default, bon::Builder)]
pub struct ApplicationConfig {
    // Application-specific settings can be added here
    // For example: log_level, theme, window settings, etc.
}
