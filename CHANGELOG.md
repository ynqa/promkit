# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.10.0] - 2025-01-14

### Added
- **Async support**: Full async/await pattern implementation for better performance and responsiveness
- **Singleton EventStream**: Prevents cursor position reading errors and improves overall reliability
- **SharedRenderer**: Thread-safe rendering system with `Arc<Renderer<K>>` and `SkipMap` for efficient pane management
- **Lifecycle management**: Clear separation of `initialize`, `evaluate`, and `finalize` phases for better control flow

### Changed
- Migrated to async/await pattern throughout the codebase
- Improved rendering performance with shared renderer architecture
- Enhanced error handling and reliability for cursor position operations

### Improved
- Better thread safety with Arc-based renderer sharing
- More efficient pane management using SkipMap data structure
- Clearer application lifecycle with distinct phases

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
