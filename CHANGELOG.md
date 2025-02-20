# Changelog

All notable changes to this project will be documented in this file. See [commit-and-tag-version](https://github.com/absolute-version/commit-and-tag-version) for commit guidelines.

## [0.8.3](https://github.com/mnemnk/mnemnk-app/compare/v0.8.2...v0.8.3) (2025-02-20)


### Features

* 🎸 agents page ([#64](https://github.com/mnemnk/mnemnk-app/issues/64)) ([4185bb7](https://github.com/mnemnk/mnemnk-app/commit/4185bb7b48e520efe49d01a565e97f01a9da40a4)), closes [#61](https://github.com/mnemnk/mnemnk-app/issues/61)
* 🎸 Settings page ([#60](https://github.com/mnemnk/mnemnk-app/issues/60)) ([e658ab4](https://github.com/mnemnk/mnemnk-app/commit/e658ab4bf636f8a188f2b8a37f2a4213352efadd)), closes [#10](https://github.com/mnemnk/mnemnk-app/issues/10)

## [0.8.2](https://github.com/mnemnk/mnemnk-app/compare/v0.8.1...v0.8.2) (2025-02-17)


### Features

* 🎸 new navbar ([#54](https://github.com/mnemnk/mnemnk-app/issues/54)) ([0819a21](https://github.com/mnemnk/mnemnk-app/commit/0819a21c8d05b0d315b7210c73b32110dd747e23))
* 🎸 uninstall unused libraries ([#55](https://github.com/mnemnk/mnemnk-app/issues/55)) ([654826c](https://github.com/mnemnk/mnemnk-app/commit/654826c36c13671228e4c7ded59dc32dc5bc1049))
* 🎸 update event calendar design ([#56](https://github.com/mnemnk/mnemnk-app/issues/56)) ([6afc91c](https://github.com/mnemnk/mnemnk-app/commit/6afc91cda584f20caa5ae61438f21cc36cf31332))

## [0.8.1](https://github.com/mnemnk/mnemnk-app/compare/v0.8.0...v0.8.1) (2025-02-14)


### Features

* 🎸 close window using escape ([#52](https://github.com/mnemnk/mnemnk-app/issues/52)) ([4076e63](https://github.com/mnemnk/mnemnk-app/commit/4076e63ee3edd2e330ea390ddf346bc6bd6495ff)), closes [#51](https://github.com/mnemnk/mnemnk-app/issues/51)


### Bug Fixes

* 🐛 cannot find agents in macos ([#49](https://github.com/mnemnk/mnemnk-app/issues/49)) ([a4dc7f3](https://github.com/mnemnk/mnemnk-app/commit/a4dc7f3fbff6260ccbe111870b13926644035ffb)), closes [#46](https://github.com/mnemnk/mnemnk-app/issues/46)
* 🐛 open window by clicking the app icon in task bar (macos) ([#50](https://github.com/mnemnk/mnemnk-app/issues/50)) ([e55e345](https://github.com/mnemnk/mnemnk-app/commit/e55e34553ea340c59365cfc3cf163a6292e20b0e)), closes [#47](https://github.com/mnemnk/mnemnk-app/issues/47)

## [0.8.0](https://github.com/mnemnk/mnemnk-app/compare/v0.7.0...v0.8.0) (2025-02-13)


### ⚠ BREAKING CHANGES

* 🧨 need to install agents separately and change agent names in settings

### Features

* 🎸 remove agents code and search agents from path ([#39](https://github.com/mnemnk/mnemnk-app/issues/39)) ([02cffb7](https://github.com/mnemnk/mnemnk-app/commit/02cffb773e11087776e1522aaf17ad55ac484063)), closes [#38](https://github.com/mnemnk/mnemnk-app/issues/38)
* 🎸 set background color to applications ([#42](https://github.com/mnemnk/mnemnk-app/issues/42)) ([2de9867](https://github.com/mnemnk/mnemnk-app/commit/2de986785f659b4a945665d833b7f6c3b0c4325a))


### Bug Fixes

* 🐛 agent name in default settings ([#41](https://github.com/mnemnk/mnemnk-app/issues/41)) ([e4f76f0](https://github.com/mnemnk/mnemnk-app/commit/e4f76f027907f3eee74e6dccb8fa8fcae9b7e326))
* 🐛 missing fs crate on unix ([#40](https://github.com/mnemnk/mnemnk-app/issues/40)) ([3ed57ad](https://github.com/mnemnk/mnemnk-app/commit/3ed57adbcd48cd6b85f60255f4becb0213dc5173))
* 🐛 unused warning about use std::fs on windows ([#44](https://github.com/mnemnk/mnemnk-app/issues/44)) ([735ea12](https://github.com/mnemnk/mnemnk-app/commit/735ea12ddba5201cb16131408a134e808ce75cc4))

## [0.7.0](https://github.com/mnemnk/mnemnk-app/compare/v0.6.0...v0.7.0) (2025-02-10)


### ⚠ BREAKING CHANGES

* 🧨 mnemnk-api now requires API key

### Features

* 🎸 api key for mnemnk-api ([#37](https://github.com/mnemnk/mnemnk-app/issues/37)) ([936ca15](https://github.com/mnemnk/mnemnk-app/commit/936ca159378e9eb6cdfde7cd86b143e101ab5a1e)), closes [#3](https://github.com/mnemnk/mnemnk-app/issues/3)


### Bug Fixes

* 🐛 some of app events are not displayed because of too many ([#36](https://github.com/mnemnk/mnemnk-app/issues/36)) ([1b3aaa8](https://github.com/mnemnk/mnemnk-app/commit/1b3aaa8823a0da18b7b0e450b567b01dc32dcc06))

## [0.6.0](https://github.com/mnemnk/mnemnk-app/compare/v0.5.5...v0.6.0) (2025-02-09)


### Features

* 🎸 change default of mnemnk-application interval ([#35](https://github.com/mnemnk/mnemnk-app/issues/35)) ([e919247](https://github.com/mnemnk/mnemnk-app/commit/e9192476a100d57b4a37e87d6849e2dfd9effb6c))
* 🎸 support more frequent app updates ([#34](https://github.com/mnemnk/mnemnk-app/issues/34)) ([ff636a0](https://github.com/mnemnk/mnemnk-app/commit/ff636a098a945068e9ebd4877fc978c868b5d904))

## [0.5.5](https://github.com/mnemnk/mnemnk-app/compare/v0.5.1...v0.5.5) (2025-02-06)

### ⚠ BREAKING CHANGES

* 🧨 Applicaiton Identifier has been changed.
* 🧨 config path has been changed.

## [0.5.1](https://github.com/mnemnk/mnemnk-app/compare/v0.5.0...v0.5.1) (2025-02-01)

* chore: 🤖 npm update and cargo update ([#25](https://github.com/mnemnk/mnemnk-app/commit/5701c14c10b808345abd666587e3beadedf6bcc4))

## [0.5.0](https://github.com/mnemnk/mnemnk-app/compare/v0.4.3...v0.5.0) (2025-02-01)


### ⚠ BREAKING CHANGES

* 🧨 Applicaiton Identifier has been changed.
* 🧨 config path has been changed.

### Bug Fixes

* 🐛 change identifier from com.mnemnk.app into Mnemnk ([#19](https://github.com/mnemnk/mnemnk-app/issues/19)) ([67f3059](https://github.com/mnemnk/mnemnk-app/commit/67f305944766c6d4e8d3f77e79899072b57a9d40)), closes [#16](https://github.com/mnemnk/mnemnk-app/issues/16)
* 🐛 default-config.yml stored in rs.com.mnemnk.app in macos ([#18](https://github.com/mnemnk/mnemnk-app/issues/18)) ([02c08c1](https://github.com/mnemnk/mnemnk-app/commit/02c08c1643c863578a29dbfd7179254430370151)), closes [#17](https://github.com/mnemnk/mnemnk-app/issues/17)
* 🐛 not open window in macos ([#22](https://github.com/mnemnk/mnemnk-app/issues/22)) ([29a0940](https://github.com/mnemnk/mnemnk-app/commit/29a09405f3c8fe982fa5861cb8a8c32d7a93bc23)), closes [#15](https://github.com/mnemnk/mnemnk-app/issues/15)
* 🐛 window is not open in macos ([#20](https://github.com/mnemnk/mnemnk-app/issues/20)) ([56faed6](https://github.com/mnemnk/mnemnk-app/commit/56faed699ba9755f17c1dc9223799e7331970529))

## [0.4.3](https://github.com/mnemnk/mnemnk-app/compare/v0.4.2...v0.4.3) (2025-01-28)
