# Change Log
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/)
and this project adheres to [Semantic Versioning](https://semver.org/).

<!-- next-header -->
## [Unreleased] - ReleaseDate

## [0.12.9] - 2025-01-30

### Internal

- Update dependencies

## [0.12.8] - 2025-01-24

### Internal

- Removed a dependency

## [0.12.7] - 2024-07-25

### Compatibility

- Update MSRV to 1.74

## [0.12.6] - 2024-02-16

### Features

- Implement `Display` for `Footer`

## [0.12.5] - 2024-02-13

### Internal

- Update dependencies

## [0.12.4] - 2023-07-14

### Internal

- Update dependencies

## [0.12.3] - 2023-03-18

### Internal

- Update dependencies

## [0.12.2] - 2023-02-22

### Internal

- Update dependencies

## [0.12.1] - 2022-12-29

### Fixes

- Ensure footers value isn't confused with another footer

## [0.12.0] - 2022-07-18

### Fixes

- Error when a newline doesn't separate summary from description

## [0.11.3] - 2022-04-14

### Fixes

- Don't treat body text as a footer but require a newline

## [0.11.2] - 2022-01-18

### Fixes

- When a body and footer have extra newlines between them, don't put them at the end of the body
- Handle windows newlines (was missing footers with them)

## [0.11.1] - 2021-12-14

### Fixes

- Clarify error messages

## [0.11.0] - 2021-10-19

### Breaking Changes

- Some grammar changes *might* have made us more restrictive, but more likely they have made parsing more loose
- `FooterSeparator` variants have been renamed

### Fixes

- Parser is now closer to [the proposed grammar](https://github.com/conventional-commits/parser)

## [0.10.3] - 2021-09-18

### Fixes

- Relaxed some lifetimes, associating them with the message, rather than `Commit`.

## [0.10.2] - 2021-09-06

### Fixes

- Support scopes with numbers in them, like `x86`

## [0.10.1] - 2021-09-03

### Fixes

- Allow trailing newlines when there is no body

## [0.10.0] - 2021-08-18

### Features

- `Commit::breaking_description` to handle the two potential sources for you

### Breaking Changes

- Moved `<TYPE>` consts to `Type::<TYPE>`.

## [0.9.2] - 2021-05-24

### Features

- `serde` feature for serializing `Commit`.

## [0.9.1] - 2021-01-30

## [0.9.0] - 2020-05-06

### Breaking Changes

- Made error type work as `Send` / `Sync`.

## [0.8.0] - 2020-05-06

### Breaking Changes

- `::new` has been renamed to `::new_unchecked` to signify that it bypasses validity checks.

## [0.7.0] - 2020-05-05

### Breaking Changes

- Forked `conventional` as `git-conventional`
- Merged `Simple` and `Typed` APIs (identifiers are typed, otherwise `str`s), removing the need to pull in a trait.

### Features

- Add typed identifier equality with `str`.
- Added constants for common `type_`s.
- Made it easier to find the footer that describes a breaking change.
- Expose means to convert `str` into typed identifier with validation.

### Fixes

<!-- next-url -->
[Unreleased]: https://github.com/crate-ci/git-conventional/compare/v0.12.9...HEAD
[0.12.9]: https://github.com/crate-ci/git-conventional/compare/v0.12.8...v0.12.9
[0.12.8]: https://github.com/crate-ci/git-conventional/compare/v0.12.7...v0.12.8
[0.12.7]: https://github.com/crate-ci/git-conventional/compare/v0.12.6...v0.12.7
[0.12.6]: https://github.com/crate-ci/git-conventional/compare/v0.12.5...v0.12.6
[0.12.5]: https://github.com/crate-ci/git-conventional/compare/v0.12.4...v0.12.5
[0.12.4]: https://github.com/crate-ci/git-conventional/compare/v0.12.3...v0.12.4
[0.12.3]: https://github.com/crate-ci/git-conventional/compare/v0.12.2...v0.12.3
[0.12.2]: https://github.com/crate-ci/git-conventional/compare/v0.12.1...v0.12.2
[0.12.1]: https://github.com/crate-ci/git-conventional/compare/v0.12.0...v0.12.1
[0.12.0]: https://github.com/crate-ci/git-conventional/compare/v0.11.3...v0.12.0
[0.11.3]: https://github.com/crate-ci/git-conventional/compare/v0.11.2...v0.11.3
[0.11.2]: https://github.com/crate-ci/git-conventional/compare/v0.11.1...v0.11.2
[0.11.1]: https://github.com/crate-ci/git-conventional/compare/v0.11.0...v0.11.1
[0.11.0]: https://github.com/crate-ci/git-conventional/compare/v0.10.3...v0.11.0
[0.10.3]: https://github.com/crate-ci/git-conventional/compare/v0.10.2...v0.10.3
[0.10.2]: https://github.com/crate-ci/git-conventional/compare/v0.10.1...v0.10.2
[0.10.1]: https://github.com/crate-ci/git-conventional/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/crate-ci/git-conventional/compare/v0.9.2...v0.10.0
[0.9.2]: https://github.com/crate-ci/git-conventional/compare/v0.9.1...v0.9.2
[0.9.1]: https://github.com/crate-ci/git-conventional/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/crate-ci/git-conventional/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/crate-ci/git-conventional/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/crate-ci/git-conventional/compare/ccaed9b35854a3536c4a2c89b89e33fbc5b6b4e4...v0.7.0
