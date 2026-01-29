# Yunara Player

Core application logic library for Yunara music player.

## Purpose

This crate contains all application state, business logic, services, and UI view components for the Yunara music player. It is designed to be platform-agnostic and testable independently of the main binary.

## Structure

- `app_state.rs` - Global application state management
- `config.rs` - Application configuration structures
- `state/` - UI state and view components
  - `app_view.rs` - Main application view
  - `player_state.rs` - Player state management
- `services/` - Business logic services
- `client.rs` - API client code
- `util.rs` - Utility functions
- `consts.rs` - Application constants

## Architecture

Follows GPUI's Entity system for reactive state management. The `AppState` is the root of the application state tree, and UI components subscribe to state changes through GPUI's entity system.
