# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.0] - 2025-07-xx

### Added
- **Async support**: Full async/await pattern implementation for better performance and responsiveness
- **SharedRenderer**: Thread-safe rendering system with `Arc<Renderer<K>>` and `SkipMap` for efficient pane management
- **Lifecycle management**: Clear separation of `initialize`, `evaluate`, and `finalize` phases for better control flow through the `Prompt` trait. The actual event loop implementation demonstrates this lifecycle:
  
  ```rust
  async fn run(&mut self) -> anyhow::Result<Self::Return> {［
      // 1. Initialize phase
      self.initialize().await?;

      // 2. Event evaluation loop
      while let Some(event) = EVENT_STREAM.lock().await.next().await {
          match event {
              Ok(event) => {
                  if self.evaluate(&event).await? == Signal::Quit {
                      break; // Exit loop when quit signal received
                  }
              }
              ... /// Handle errors
          }
      }

      // 3. Finalize phase
      self.finalize()
  }
  ```
  
  **Phase breakdown:**
  - `initialize()`: Called once before entering the event loop for setup
  - `evaluate()`: Called for each event, returns `Signal::Continue` or `Signal::Quit`
  - `finalize()`: Called after loop exit to produce the final result
  - Singleton `EVENT_STREAM` prevents cursor position read errors across multiple prompts
- **Spinner widget**: New widget for displaying spinner animations during async task execution
  - `spinner::State` trait: Interface for checking idle state asynchronously
  - `spinner::run` function: Executes frame-based spinner animations
  - `spinner::frame` module: Provides various spinner frame patterns
- **BYOP (Build Your Own Preset) example**: Custom prompt implementation example
  - Integration demo of spinner and text editor
  - UI state management during async task execution
  - Task start, completion, and cancellation functionality

### Changed
- Migrated to async/await pattern throughout the codebase
- Improved rendering performance with shared renderer architecture

### Improved
- Better thread safety with Arc-based renderer sharing
- More efficient pane management using SkipMap data structure
- Clearer application lifecycle with distinct phases
- Better patterns and best practices for async task management
- Enhanced user experience with spinner animations

### Technical Details
- Introduced `Arc<Renderer<K>>` for thread-safe renderer sharing
- Implemented `SkipMap` for efficient pane management
- Added singleton pattern for EventStream to prevent conflicts
- Restructured application flow with initialize → evaluate → finalize phases

---

## Previous Versions

For versions prior to 0.10.0, please refer to the git history or GitHub releases.

[Unreleased]: https://github.com/ynqa/promkit/compare/v0.10.0...HEAD
[0.10.0]: https://github.com/ynqa/promkit/releases/tag/v0.10.0
