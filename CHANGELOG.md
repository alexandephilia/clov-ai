# Changelog

## [0.35.0](https://github.com/alexandephilia/clov-ai/compare/v0.34.4...v0.35.0) (2026-03-08)


### Features

* **scan:** add support for multiple glob patterns and the OR (-o) operator ([17c0173](https://github.com/alexandephilia/clov-ai/commit/17c0173b82950863f3feb3cd87f2bb3b145f9e70))

## [0.34.4](https://github.com/alexandephilia/clov-ai/compare/v0.34.3...v0.34.4) (2026-03-08)


### Bug Fixes

* **pulse:** use modern minimal separators instead of full boxes ([10fe9e6](https://github.com/alexandephilia/clov-ai/commit/10fe9e6bfe6499b7f0aab1808e795710f77250b9))

## [0.34.3](https://github.com/alexandephilia/clov-ai/compare/v0.34.2...v0.34.3) (2026-03-08)


### Bug Fixes

* **pulse:** replace misaligned 3-column grid with individual box-drawn cards ([436273e](https://github.com/alexandephilia/clov-ai/commit/436273e6b39fea411904712a45a5e55ba4388464))

## [0.34.2](https://github.com/alexandephilia/clov-ai/compare/v0.34.1...v0.34.2) (2026-03-08)


### Bug Fixes

* **ci:** update benchmark script for current CLI ([b0255ab](https://github.com/alexandephilia/clov-ai/commit/b0255abd577c4993fdd18d94ce1883dc1795921d))

## [0.34.1](https://github.com/alexandephilia/clov-ai/compare/v0.34.0...v0.34.1) (2026-03-08)


### Bug Fixes

* **cli:** require command for passthrough ([c266916](https://github.com/alexandephilia/clov-ai/commit/c266916b0b5960d4b645f8314916af28cf92f87d))

## [0.34.0](https://github.com/alexandephilia/clov-ai/compare/v0.33.0...v0.34.0) (2026-03-08)


### Features

* **pulse:** rename global summary title ([c24887b](https://github.com/alexandephilia/clov-ai/commit/c24887ba43f426100672f5bfc82bdfc20ff3a8ec))
* **pulse:** switch to command grid layout ([572dd08](https://github.com/alexandephilia/clov-ai/commit/572dd0848710fed1dc7a77ecdaaa333a5303f387))

## [0.33.0](https://github.com/alexandephilia/clov-ai/compare/v0.32.0...v0.33.0) (2026-03-08)


### Features

* **pulse:** redesign savings output with tactical blocks ([f130ec8](https://github.com/alexandephilia/clov-ai/commit/f130ec86580bf7107b63ca83c29210d1859dbf93))

## [0.32.0](https://github.com/alexandephilia/clov-ai/compare/v0.31.0...v0.32.0) (2026-03-08)


### Features

* **cli:** remove legacy command aliases ([944d963](https://github.com/alexandephilia/clov-ai/commit/944d9639f069bc3c968989481d114e488c961209))

## [0.31.0](https://github.com/alexandephilia/clov-ai/compare/v0.30.0...v0.31.0) (2026-03-08)


### Features

* **cli:** rename primary clov command surface ([e9bc0e5](https://github.com/alexandephilia/clov-ai/commit/e9bc0e5d9c4ed581adc79ae5a62ea106e83c7dc5))

## [0.30.0](https://github.com/alexandephilia/clov-ai/compare/v0.29.5...v0.30.0) (2026-03-08)


### Features

* **config:** add MCP presets and persistent defaults ([2891b96](https://github.com/alexandephilia/clov-ai/commit/2891b96569603037ce05897704486df452d998f6))
* **tokenizer:** add profile-aware MCP token budgeting ([a8f77ac](https://github.com/alexandephilia/clov-ai/commit/a8f77ac9f8846c607ac2af6139cadc596af1e850))
* **tracking:** add MCP parse metadata and fallback telemetry ([64a648e](https://github.com/alexandephilia/clov-ai/commit/64a648ec9d87cc57c4a75e1cfcd590ea362cd7ab))

## [0.29.5](https://github.com/alexandephilia/clov-ai/compare/v0.29.4...v0.29.5) (2026-03-07)


### Bug Fixes

* add dynamic mcp filter controls ([3bb7942](https://github.com/alexandephilia/clov-ai/commit/3bb79423ac9c1063b11adb0ad2305f121d84b5f9))

## [0.29.4](https://github.com/alexandephilia/clov-ai/compare/v0.29.3...v0.29.4) (2026-03-07)


### Bug Fixes

* clean exa quoted article artifacts ([672a140](https://github.com/alexandephilia/clov-ai/commit/672a140261530d114b8fd16fa9d5282f359de4ef))

## [0.29.3](https://github.com/alexandephilia/clov-ai/compare/v0.29.2...v0.29.3) (2026-03-07)


### Bug Fixes

* preserve long-form MCP article content ([6d73a88](https://github.com/alexandephilia/clov-ai/commit/6d73a887a210a71883be2abfddff8daec88fa443))
* strip search provider noise keys from MCP web search responses ([d307a92](https://github.com/alexandephilia/clov-ai/commit/d307a92edbf8191d1c50dbd1e77dc2f501cf5853))

## [0.29.2](https://github.com/alexandephilia/clov-ai/compare/v0.29.1...v0.29.2) (2026-03-08)

### Bug Fixes

* fix CI release workflow — DEB/RPM non-blocking, bash glob crash on missing packages, cargo install replaces broken taiki-e action tags

## [0.29.1](https://github.com/alexandephilia/clov-ai/compare/v0.29.0...v0.29.1) (2026-03-08)

### Features

* harden MCP proxy framing and structured response filtering ([edf0999](https://github.com/alexandephilia/clov-ai/commit/edf0999))

## [0.29.0](https://github.com/alexandephilia/clov-ai/compare/v0.28.0...v0.29.0) (2026-03-07)


### Features

* optimize release workflow with pre-built packaging tools ([611fd80](https://github.com/alexandephilia/clov-ai/commit/611fd80c505614afc9d9a0fe8d2c0d6011b2dabf))

## [0.28.0](https://github.com/alexandephilia/clov-ai/compare/v0.27.2...v0.28.0) (2026-03-07)


### Features

* implement Universal MCP filtering and generalized documentation (v0.27.7) ([d010ba4](https://github.com/alexandephilia/clov-ai/commit/d010ba491dcf2dee813dbdebb79c7957e8746065))

## [0.27.7](https://github.com/alexandephilia/clov-ai/compare/v0.27.2...v0.27.7) (2026-03-08)

### Features

- implement Universal MCP filtering (detects search, code, data structures)
- generalize documentation to reflect provider-agnostic MCP architecture

## [0.27.2](https://github.com/alexandephilia/clov-ai/compare/v0.27.0...v0.27.2) (2026-03-07)

### Features

- implement Universal MCP filtering (detects search, code, data structures)

### Maintenance

- align release metadata and Homebrew formula with v0.27.2

## [0.27.0](https://github.com/alexandephilia/clov-ai/compare/v0.26.4...v0.27.0) (2026-03-07)

### Features

- implement Universal MCP filtering (detects search, code, data structures) (v0.26.5) ([13de67e](https://github.com/alexandephilia/clov-ai/commit/13de67ed8f7606c449734a9bb64fbd9d99b7ff52))

## [0.26.4](https://github.com/alexandephilia/clov-ai/compare/v0.26.3...v0.26.4) (2026-03-07)

### Bug Fixes

- repair release automation ([703bbba](https://github.com/alexandephilia/clov-ai/commit/703bbba0645451d5714cc5cc13f369c38f9fa7c0))
- update CLOV.md awareness template ([dd199b1](https://github.com/alexandephilia/clov-ai/commit/dd199b1f542727c1cc2d16f19387154a4ffe31dd))
