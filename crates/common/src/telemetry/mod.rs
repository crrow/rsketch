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

//! # Telemetry Module
//!
//! This module provides comprehensive telemetry capabilities for the rsketch
//! application, including structured logging, distributed tracing, panic
//! handling, and OpenTelemetry integration. It is designed to provide
//! observability across distributed systems and help with debugging,
//! monitoring, and performance analysis.
//!
//! ## Overview
//!
//! The telemetry module consists of four main components:
//!
//! - **[`logging`]**: Configurable logging system with support for file
//!   rotation, JSON/text formatting, and OpenTelemetry integration
//! - **[`panic_hook`]**: Enhanced panic handling with structured logging,
//!   backtraces, and optional deadlock detection
//! - **[`tracing_context`]**: Distributed tracing context management for
//!   propagating trace information across service boundaries
//! - **[`tracing_sampler`]**: Intelligent sampling strategies to control the
//!   volume of telemetry data collected
//!
//! ## Quick Start
//!
//! ### Basic Setup
//!
//! For simple applications with default settings:
//!
//! ```rust
//! use rsketch_common::telemetry::{logging::init_tracing_subscriber, panic_hook::set_panic_hook};
//!
//! // Initialize basic logging to stdout
//! let _guards = init_tracing_subscriber("my-app");
//!
//! // Set up enhanced panic handling
//! set_panic_hook();
//!
//! // Your application code here
//! tracing::info!("Application started");
//! ```
//!
//! ### Advanced Configuration
//!
//! For production environments with file logging and OpenTelemetry:
//!
//! ```rust,no_run
//! use std::collections::HashMap;
//!
//! use rsketch_common::telemetry::{
//!     logging::{LogFormat, LoggingOptions, OtlpExportProtocol, init_global_logging},
//!     panic_hook::set_panic_hook,
//!     tracing_sampler::TracingSampleOptions,
//! };
//!
//! // Configure comprehensive logging
//! let logging_opts = LoggingOptions {
//!     dir:                  "/var/log/myapp".to_string(),
//!     level:                Some("info,hyper=warn".to_string()),
//!     log_format:           LogFormat::Json,
//!     enable_otlp_tracing:  true,
//!     otlp_endpoint:        Some("http://jaeger:14268/api/traces".to_string()),
//!     otlp_export_protocol: Some(OtlpExportProtocol::Http),
//!     tracing_sample_ratio: Some(TracingSampleOptions {
//!         default_ratio: 0.1,
//!         rules:         vec![],
//!     }),
//!     append_stdout:        true,
//!     max_log_files:        100,
//!     otlp_headers:         HashMap::new(),
//! };
//!
//! let _guards = init_global_logging(
//!     "my-service",
//!     &logging_opts,
//!     &Default::default(),
//!     Some("node-001".to_string()),
//! );
//!
//! set_panic_hook();
//!
//! tracing::info!("Service started with full telemetry");
//! ```
//!
//! ## Features
//!
//! ### Structured Logging
//!
//! - **Multiple Outputs**: Simultaneous logging to stdout, files, and
//!   error-specific files
//! - **Flexible Formats**: Support for both human-readable text and
//!   machine-parseable JSON
//! - **Log Rotation**: Automatic hourly rotation with configurable retention
//! - **Level Filtering**: Fine-grained control over log levels per module
//! - **Runtime Reload**: Dynamic log level changes without restart
//!
//! ### Distributed Tracing
//!
//! - **W3C Compliance**: Full support for W3C Trace Context standard
//! - **Cross-Service**: Propagate traces across service boundaries
//! - **OpenTelemetry**: Native integration with OpenTelemetry ecosystem
//! - **Flexible Export**: Support for both gRPC and HTTP export protocols
//! - **Custom Headers**: Configurable headers for authentication and routing
//!
//! ### Intelligent Sampling
//!
//! - **Adaptive Rates**: Different sampling rates for different operations
//! - **Protocol-Aware**: Custom sampling rules based on protocol and request
//!   type
//! - **Performance**: Minimize overhead while maintaining observability
//! - **Configurable**: Runtime-configurable sampling strategies
//!
//! ### Enhanced Error Handling
//!
//! - **Structured Panics**: Detailed panic information with context
//! - **Backtraces**: Full stack traces for debugging
//! - **Metrics Integration**: Panic counters for monitoring
//! - **Deadlock Detection**: Optional detection of threading issues
//!   (feature-gated)
//!
//! ## Architecture
//!
//! The telemetry system is built on top of the Rust tracing ecosystem:
//!
//! ```text
//! ┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
//! │   Application   │───▶│   Tracing Core   │───▶│   Subscribers   │
//! │     Code        │    │                  │    │                 │
//! └─────────────────┘    └──────────────────┘    └─────────────────┘
//!                                │                         │
//!                                ▼                         ▼
//!                        ┌──────────────┐         ┌──────────────┐
//!                        │   Filtering  │         │   Exporters  │
//!                        │   & Sampling │         │              │
//!                        └──────────────┘         └──────────────┘
//!                                                          │
//!                                                          ▼
//!                                                  ┌──────────────┐
//!                                                  │   Outputs    │
//!                                                  │ • Files      │
//!                                                  │ • Stdout     │
//!                                                  │ • OpenTel    │
//!                                                  └──────────────┘
//! ```
//!
//! ## Configuration
//!
//! The logging system supports environment-based configuration via RUST_LOG
//! and OTEL_* variables. Programmatic configuration uses LoggingOptions for
//! detailed control over outputs, formats, and OpenTelemetry settings.
//!
//! ## Performance Considerations
//!
//! Sampling reduces overhead in high-throughput scenarios with configurable
//! ratios per protocol and request type. Non-blocking writers prevent I/O
//! blocking. Hourly log rotation controls disk usage.
//!
//! ## Integration
//!
//! Supports OpenTelemetry Protocol for integration with Jaeger, Tempo, and
//! other observability backends. Prometheus metrics are automatically exposed
//! for panic monitoring.
//!
//! ## Best Practices
//!
//! Initialize telemetry early, use structured logging, implement sampling for
//! high-throughput scenarios, propagate tracing context across services, and
//! keep worker guards alive for the application lifetime.
//!
//! ## Troubleshooting
//!
//! Common issues include OTLP connectivity problems, high memory usage from
//! excessive sampling, missing context in async code, and file permission
//! errors. Enable debug logging with RUST_LOG for detailed diagnostics.
//!
//! ## Feature Flags
//!
//! Optional features include `tokio-console` for async debugging and
//! `deadlock_detection` for runtime deadlock monitoring.

pub mod logging;
pub mod panic_hook;
pub mod tracing_context;
pub mod tracing_sampler;
