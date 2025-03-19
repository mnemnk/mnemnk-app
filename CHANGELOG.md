# Changelog

All notable changes to this project will be documented in this file. See [commit-and-tag-version](https://github.com/absolute-version/commit-and-tag-version) for commit guidelines.

## [0.11.0](https://github.com/mnemnk/mnemnk-app/compare/v0.10.1...v0.11.0) (2025-03-19)


### ⚠ BREAKING CHANGES

* 🧨 message passing based on edges
* 🧨 configuration file is totally changed.

### Features

* 🎸 add board node ([#96](https://github.com/mnemnk/mnemnk-app/issues/96)) ([403a050](https://github.com/mnemnk/mnemnk-app/commit/403a0509d938bc5704622dfb5edd01e2b6a83727))
* 🎸 agent configuration by json files ([#107](https://github.com/mnemnk/mnemnk-app/issues/107)) ([a54d000](https://github.com/mnemnk/mnemnk-app/commit/a54d0008c11c68654f4d1d0de35ea3f15ea9e6fb))
* 🎸 Agents page with SvelteFlow ([#94](https://github.com/mnemnk/mnemnk-app/issues/94)) ([04fa0fd](https://github.com/mnemnk/mnemnk-app/commit/04fa0fd169cba9d92e3c5c85bffd8ad558ad436c))
* 🎸 change agent protocols ([#112](https://github.com/mnemnk/mnemnk-app/issues/112)) ([132e5bd](https://github.com/mnemnk/mnemnk-app/commit/132e5bd97c97a509249d4b26e6d4fa4533a74f4e))
* 🎸 message passing based on edges ([#100](https://github.com/mnemnk/mnemnk-app/issues/100)) ([f2ea902](https://github.com/mnemnk/mnemnk-app/commit/f2ea9020e986b4d1bb34fd65c432c7bb59aef0ef))
* 🎸 move agent config into directory ([#110](https://github.com/mnemnk/mnemnk-app/issues/110)) ([e8f0423](https://github.com/mnemnk/mnemnk-app/commit/e8f0423e1a65a6e5250897d5617bfa54c595d802))
* 🎸 save and restore flow edges ([#97](https://github.com/mnemnk/mnemnk-app/issues/97)) ([a2ddcb8](https://github.com/mnemnk/mnemnk-app/commit/a2ddcb88c47845e6f07c31c1c56d4516e24e8309))
* 🎸 separate agent configs and flows from settings ([#103](https://github.com/mnemnk/mnemnk-app/issues/103)) ([f0d09ba](https://github.com/mnemnk/mnemnk-app/commit/f0d09ba929d130daee1a1dbc2d01bcdb087d1f27))
* 🎸 start agent from agent dir ([#111](https://github.com/mnemnk/mnemnk-app/issues/111)) ([0ca2c9b](https://github.com/mnemnk/mnemnk-app/commit/0ca2c9b75c452a102aee240d7f46cf572843988d))
* 🎸 update agent status without restarting app ([#101](https://github.com/mnemnk/mnemnk-app/issues/101)) ([43c75f9](https://github.com/mnemnk/mnemnk-app/commit/43c75f986c7284f149ea3234844bd1ef6e70bbef))
* 🎸 update monitor page ([#102](https://github.com/mnemnk/mnemnk-app/issues/102)) ([d388a4e](https://github.com/mnemnk/mnemnk-app/commit/d388a4e27c395292aacc1ebf3b4917232dd4a66e))
* 🎸 use SvelteFlow node handle as kind name ([#105](https://github.com/mnemnk/mnemnk-app/issues/105)) ([dfd9fec](https://github.com/mnemnk/mnemnk-app/commit/dfd9fecd2e1dc8f55f3f00bc93a74f830cb9a05b))
* Monitor page ([4121da7](https://github.com/mnemnk/mnemnk-app/commit/4121da744838fb5081b09952e4357bf9cd43d0a8))


### Bug Fixes

* 🐛 remove error log for blank board name ([#109](https://github.com/mnemnk/mnemnk-app/issues/109)) ([fd27a9c](https://github.com/mnemnk/mnemnk-app/commit/fd27a9c49ff1c7ace2077988304f454a1884d03d))
* 🐛 remove find_agent_node_mut ([#104](https://github.com/mnemnk/mnemnk-app/issues/104)) ([efc6b86](https://github.com/mnemnk/mnemnk-app/commit/efc6b869d3fda2be2ce78edccd14118fe5445ecf))
* 🐛 remove link to config.yml ([#113](https://github.com/mnemnk/mnemnk-app/issues/113)) ([bfab127](https://github.com/mnemnk/mnemnk-app/commit/bfab1273462ff6ef10ad30bf89b59d786173aaaf))
* 🐛 remove unused page.ts from monitor ([#93](https://github.com/mnemnk/mnemnk-app/issues/93)) ([ae100a6](https://github.com/mnemnk/mnemnk-app/commit/ae100a62933026de05fc226a0261ef60a2e18d00))
* 🐛 save and restore agents ([#95](https://github.com/mnemnk/mnemnk-app/issues/95)) ([15f2b85](https://github.com/mnemnk/mnemnk-app/commit/15f2b85c781e498cf15b344f449db095698ca52b))

## [0.10.1](https://github.com/mnemnk/mnemnk-app/compare/v0.10.0...v0.10.1) (2025-03-07)

## [0.10.0](https://github.com/mnemnk/mnemnk-app/compare/v0.9.2...v0.10.0) (2025-03-07)


### ⚠ BREAKING CHANGES

* 🧨 all commands must start with dot
* 🧨 update icons

### Features

* 🎸 add dot prefix to command names ([#85](https://github.com/mnemnk/mnemnk-app/issues/85)) ([637e619](https://github.com/mnemnk/mnemnk-app/commit/637e61975896ce5c6d26da5e051402ff7b650d72)), closes [#81](https://github.com/mnemnk/mnemnk-app/issues/81)
* 🎸 update icons ([#82](https://github.com/mnemnk/mnemnk-app/issues/82)) ([134a0ff](https://github.com/mnemnk/mnemnk-app/commit/134a0ff65832c6375d9494dee0a278d9a0a61270)), closes [#48](https://github.com/mnemnk/mnemnk-app/issues/48)


### Bug Fixes

* 🐛 unsubscribe agent on terminate ([#84](https://github.com/mnemnk/mnemnk-app/issues/84)) ([aa727a5](https://github.com/mnemnk/mnemnk-app/commit/aa727a585aeb5ccc28abf585f5bec398fc494b1a)), closes [#83](https://github.com/mnemnk/mnemnk-app/issues/83)

## [0.9.2](https://github.com/mnemnk/mnemnk-app/compare/v0.9.1...v0.9.2) (2025-03-03)


### Features

* 🎸 day start hour setting for daily page ([#80](https://github.com/mnemnk/mnemnk-app/issues/80)) ([2c6df12](https://github.com/mnemnk/mnemnk-app/commit/2c6df1237b97a630214c85938b0b66c689fc8420)), closes [#79](https://github.com/mnemnk/mnemnk-app/issues/79)

## [0.9.1](https://github.com/mnemnk/mnemnk-app/compare/v0.9.0...v0.9.1) (2025-02-26)


### Features

* 🎸 shortcut keys for search ([#78](https://github.com/mnemnk/mnemnk-app/issues/78)) ([8ed5d3f](https://github.com/mnemnk/mnemnk-app/commit/8ed5d3f79467116f07bbf0f706fc97152ec12576))
* 🎸 shortcuts for screenshot only and fullscreen ([#77](https://github.com/mnemnk/mnemnk-app/issues/77)) ([f2ebb49](https://github.com/mnemnk/mnemnk-app/commit/f2ebb49d1e632c65f456fb227602571e673d8fe5))

## [0.9.0](https://github.com/mnemnk/mnemnk-app/compare/v0.8.5...v0.9.0) (2025-02-25)


### ⚠ BREAKING CHANGES

* 🧨 Rebuiding the search index is necessary

### Features

* 🎸 custom tokenizer ([#75](https://github.com/mnemnk/mnemnk-app/issues/75)) ([b75a1e3](https://github.com/mnemnk/mnemnk-app/commit/b75a1e35ad39ed5f5456668ee4df204191358956))

## [0.8.5](https://github.com/mnemnk/mnemnk-app/compare/v0.8.4...v0.8.5) (2025-02-24)


### Features

* 🎸 index page caching ([#69](https://github.com/mnemnk/mnemnk-app/issues/69)) ([e1d90fe](https://github.com/mnemnk/mnemnk-app/commit/e1d90fe2d12c8d894c9e60a7ff55dee53f0bca5c)), closes [#63](https://github.com/mnemnk/mnemnk-app/issues/63)
* 🎸 save agent settings from agent page ([#72](https://github.com/mnemnk/mnemnk-app/issues/72)) ([c8d3a6d](https://github.com/mnemnk/mnemnk-app/commit/c8d3a6d4b02f1e8840e11cf109856d7e7562e5bd)), closes [#66](https://github.com/mnemnk/mnemnk-app/issues/66)
* 🎸 support CONFIG_SCHEMA command ([#71](https://github.com/mnemnk/mnemnk-app/issues/71)) ([7459fdd](https://github.com/mnemnk/mnemnk-app/commit/7459fdd80fa7a2b41c2e9c794f90f47d96bdfb49))


### Bug Fixes

* 🐛 release build failure on daily page ([#74](https://github.com/mnemnk/mnemnk-app/issues/74)) ([37311e6](https://github.com/mnemnk/mnemnk-app/commit/37311e6a6113c02f0a1715038527ece7041b0982))
* 🐛 validate agent name and settings ([#73](https://github.com/mnemnk/mnemnk-app/issues/73)) ([5f0e426](https://github.com/mnemnk/mnemnk-app/commit/5f0e4268d362118a3ed6d785b64be95c86c0b56d))

## [0.8.4](https://github.com/mnemnk/mnemnk-app/compare/v0.8.3...v0.8.4) (2025-02-22)


### Features

* 🎸 event scrollbar in daily page ([#67](https://github.com/mnemnk/mnemnk-app/issues/67)) ([1d8d133](https://github.com/mnemnk/mnemnk-app/commit/1d8d1330031b7976910a45dd2ece0a3f08032ea2))

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
