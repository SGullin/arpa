# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- This changelog.

### Changed
- Improved pipeline versatility.

### Removed
- Relience on rawfile storage.
- Unnecessary `read_raw_file` function.

### Fixed
- `Path` usage relating to future send-safety.
- Running the pipeline with a missing path to or installation of `psrchive` will throw an error and no longer panic.

## [0.3.2]
- Non empty `stderr` from `psrchive` tool triggers an error, instead of just a warning.

## [0.3.1]
- Removed complicating use of config module.

## [0.3.0]
- Prepared as library.
