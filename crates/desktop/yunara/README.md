# Yunara Desktop

Main binary crate for Yunara music player desktop application.

## Purpose

This crate contains only the platform-specific startup code and initialization logic. All application logic, state management, and UI components are in the `yunara-player` crate.

## Structure

- `main.rs` - Application entry point and platform initialization
- `helper.rs` - Startup helper functions (paths, logging, crash handling)
- `build.rs` - Build-time configuration

## Dependencies

- `yunara-player` - Core application logic
- `yunara-ui` - UI components and theme system
- `yunara-store` - Database and storage
- `yunara-assets` - Embedded assets
- `yunara-paths` - Path utilities
