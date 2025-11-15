# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.3](https://github.com/lmmx/syncdoc/compare/syncdoc-v0.3.2...syncdoc-v0.3.3) - 2025-11-15

### <!-- 4 -->Documentation

- update to mention inline paths CLI flag

### <!-- 8 -->Styling

- default to not injecting inline path (set in TOML) ([#42](https://github.com/lmmx/syncdoc/pull/42))

## [0.3.2](https://github.com/lmmx/syncdoc/compare/syncdoc-v0.3.1...syncdoc-v0.3.2) - 2025-11-15

### <!-- 5 -->Refactor

- tighten up Cargo manifest dir acquisition ([#41](https://github.com/lmmx/syncdoc/pull/41))

## [0.3.1](https://github.com/lmmx/syncdoc/compare/syncdoc-v0.3.0...syncdoc-v0.3.1) - 2025-11-15

### <!-- 1 -->Features

- migrate flag (implies cut + add + touch) ([#40](https://github.com/lmmx/syncdoc/pull/40))

## [0.3.0](https://github.com/lmmx/syncdoc/compare/syncdoc-v0.2.3...syncdoc-v0.3.0) - 2025-11-15

### <!-- 9 -->Other

- updated the following local packages: syncdoc-core, syncdoc-migrate

## [0.2.3](https://github.com/lmmx/syncdoc/compare/syncdoc-v0.2.2...syncdoc-v0.2.3) - 2025-11-15

### <!-- 9 -->Other

- touch missing files ([#36](https://github.com/lmmx/syncdoc/pull/36))

## [0.2.2](https://github.com/lmmx/syncdoc/compare/syncdoc-v0.2.1...syncdoc-v0.2.2) - 2025-11-15

### <!-- 4 -->Documentation

- note on CLI

## [0.1.0] - 2024-01-01

### Added
- Initial release
- `syncdoc` macro for single-item documentation injection
- `omnidoc` macro for module-wide documentation injection
- Support for functions, impl blocks, and nested modules
