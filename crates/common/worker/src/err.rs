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

use snafu::Snafu;

/// Errors that can occur when parsing cron expressions.
///
/// # Example
///
/// ```rust
/// use rsketch_common_worker::CronParseError;
///
/// let result = croner::Cron::new("invalid cron").parse();
/// match result {
///     Ok(cron) => println!("Valid cron"),
///     Err(e) => println!("Invalid cron: {}", e),
/// }
/// ```
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum CronParseError {
    /// The cron expression could not be parsed.
    ///
    /// This typically occurs when:
    /// - Invalid syntax (e.g., "60 * * * *" for minute)
    /// - Wrong number of fields (expects 5: minute hour day month weekday)
    /// - Invalid range values
    #[snafu(display("Failed to parse cron expression: {source}"))]
    InvalidExpression {
        source: croner::errors::CronError,
        #[snafu(implicit)]
        loc: snafu::Location,
    },
}
