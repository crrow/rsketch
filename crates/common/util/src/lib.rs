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

// Ensure exactly one release channel feature is enabled
#[cfg(all(feature = "release-dev", feature = "release-preview"))]
compile_error!("Features `release-dev` and `release-preview` are mutually exclusive");

#[cfg(all(feature = "release-dev", feature = "release-stable"))]
compile_error!("Features `release-dev` and `release-stable` are mutually exclusive");

#[cfg(all(feature = "release-preview", feature = "release-stable"))]
compile_error!("Features `release-preview` and `release-stable` are mutually exclusive");

#[cfg(not(any(
    feature = "release-dev",
    feature = "release-preview",
    feature = "release-stable"
)))]
compile_error!("One of `release-dev`, `release-preview`, or `release-stable` must be enabled");

pub mod crashes;
pub mod ensure_single_instance;
pub mod version;
