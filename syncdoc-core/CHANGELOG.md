# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.3](https://github.com/lmmx/syncdoc/compare/syncdoc-core-v0.3.2...syncdoc-core-v0.3.3) - 2025-11-16

### <!-- 2 -->Bug Fixes

- prevent pushing with unstaged changes post-prepush (would cause release to fail)

### <!-- 9 -->Other

- parse trait methods ([#46](https://github.com/lmmx/syncdoc/pull/46))

## [0.3.2](https://github.com/lmmx/syncdoc/compare/syncdoc-core-v0.3.1...syncdoc-core-v0.3.2) - 2025-11-15

### <!-- 9 -->Other

- get struct variants? ([#43](https://github.com/lmmx/syncdoc/pull/43))

## [0.3.1](https://github.com/lmmx/syncdoc/compare/syncdoc-core-v0.3.0...syncdoc-core-v0.3.1) - 2025-11-15

### <!-- 5 -->Refactor

- tighten up Cargo manifest dir acquisition ([#41](https://github.com/lmmx/syncdoc/pull/41))

## [0.3.0](https://github.com/lmmx/syncdoc/compare/syncdoc-core-v0.2.1...syncdoc-core-v0.3.0) - 2025-11-15

### <!-- 2 -->Bug Fixes

- inject trait docstr

## [0.1.0] - 2024-01-01

### Added
- Initial release
- `syncdoc` macro for single-item documentation injection
- `omnidoc` macro for module-wide documentation injection
- Support for functions, impl blocks, and nested modules
