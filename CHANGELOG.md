# Changelog

All notable changes to this project will be documented in this file. See [commit-and-tag-version](https://github.com/absolute-version/commit-and-tag-version) for commit guidelines.

## [0.14.4](https://github.com/mnemnk/mnemnk-app/compare/v0.14.3...v0.14.4) (2025-04-13)


### Features

* 🎸 agents list in splitpanes ([#248](https://github.com/mnemnk/mnemnk-app/issues/248)) ([660ad7c](https://github.com/mnemnk/mnemnk-app/commit/660ad7c30b6b8de6c2e5360b4ce9270bd24beadb))
* 🎸 flow list ([#250](https://github.com/mnemnk/mnemnk-app/issues/250)) ([206e45a](https://github.com/mnemnk/mnemnk-app/commit/206e45a58b6048d62ad826c795d0043ee51d12b2))
* 🎸 node context menu ([#247](https://github.com/mnemnk/mnemnk-app/issues/247)) ([62782be](https://github.com/mnemnk/mnemnk-app/commit/62782bedb4a8311c976698d9cfc1a0323cd84000))


### Bug Fixes

* 🐛 downgrade Svelte for ownership_invalid_binding warning ([#246](https://github.com/mnemnk/mnemnk-app/issues/246)) ([0e3eca1](https://github.com/mnemnk/mnemnk-app/commit/0e3eca17bbee4481cdf869d9b0cc853aee815f90))

## [0.14.3](https://github.com/mnemnk/mnemnk-app/compare/v0.14.2...v0.14.3) (2025-04-11)


### Features

* 🎸 Counter agent ([#230](https://github.com/mnemnk/mnemnk-app/issues/230)) ([e706d1c](https://github.com/mnemnk/mnemnk-app/commit/e706d1cac7924f11bd97b3bfab37e010cf4f6489))
* 🎸 display errors from agent ([#240](https://github.com/mnemnk/mnemnk-app/issues/240)) ([2dd9b35](https://github.com/mnemnk/mnemnk-app/commit/2dd9b35926c79ef41b621edf0c3334b68668d90d)), closes [#238](https://github.com/mnemnk/mnemnk-app/issues/238)


### Bug Fixes

* 🐛 flow name state ([#236](https://github.com/mnemnk/mnemnk-app/issues/236)) ([03abfd1](https://github.com/mnemnk/mnemnk-app/commit/03abfd11c103d25ce792ef562fc4f59ac93ab9c6))
* 🐛 insert agent flow on sync ([#234](https://github.com/mnemnk/mnemnk-app/issues/234)) ([a6bf5e8](https://github.com/mnemnk/mnemnk-app/commit/a6bf5e894d808614d5bc32b30ac6d87384691670))
* 🐛 preserve agent flows ([#231](https://github.com/mnemnk/mnemnk-app/issues/231)) ([4e847b9](https://github.com/mnemnk/mnemnk-app/commit/4e847b9cc4699b0371702df00ee04526727cabe3))
* 🐛 prevent reload in agent page ([#235](https://github.com/mnemnk/mnemnk-app/issues/235)) ([93a9fba](https://github.com/mnemnk/mnemnk-app/commit/93a9fba93d280e664805baadf30d96f38cb539a0))
* 🐛 prevent to add duplicated edges ([#232](https://github.com/mnemnk/mnemnk-app/issues/232)) ([7ee048c](https://github.com/mnemnk/mnemnk-app/commit/7ee048ce6ecf3afd4d18736abf38a44d38edb742))
* 🐛 remove console.log ([#233](https://github.com/mnemnk/mnemnk-app/issues/233)) ([0da8019](https://github.com/mnemnk/mnemnk-app/commit/0da8019dd358309f792df34c44fa0ae7ae6a89b8))
* 🐛 remove unused modal ([#241](https://github.com/mnemnk/mnemnk-app/issues/241)) ([27f7696](https://github.com/mnemnk/mnemnk-app/commit/27f7696324042062d38afa1f04f7ba4707cf184a))

## [0.14.2](https://github.com/mnemnk/mnemnk-app/compare/v0.14.1...v0.14.2) (2025-04-10)


### Bug Fixes

* 🐛 agents globa_config in settings page ([#228](https://github.com/mnemnk/mnemnk-app/issues/228)) ([53de30e](https://github.com/mnemnk/mnemnk-app/commit/53de30e3d42c84a0805a3a05651cfa48ee4874a8))
* 🐛 tx locks ([#226](https://github.com/mnemnk/mnemnk-app/issues/226)) ([4110d31](https://github.com/mnemnk/mnemnk-app/commit/4110d31e607dbffb21fcfb3772e609bfae579830))
* 🐛 wildcard target handle ([#227](https://github.com/mnemnk/mnemnk-app/issues/227)) ([1bab15d](https://github.com/mnemnk/mnemnk-app/commit/1bab15ddb4cf964902afc1d2f2b9e8ec8f72bfa5))

## [0.14.1](https://github.com/mnemnk/mnemnk-app/compare/v0.14.0...v0.14.1) (2025-04-09)


### Features

* 🎸 copy and paste nodes and edges ([#216](https://github.com/mnemnk/mnemnk-app/issues/216)) ([ce5be32](https://github.com/mnemnk/mnemnk-app/commit/ce5be3278e843f0a879493da0e9390dc2e39e4e7))
* 🎸 graceful shutdown database ([#221](https://github.com/mnemnk/mnemnk-app/issues/221)) ([bc6d0d4](https://github.com/mnemnk/mnemnk-app/commit/bc6d0d43d92456004dc6d424e0522bd3115f2c6e)), closes [#211](https://github.com/mnemnk/mnemnk-app/issues/211)
* 🎸 remove Monitor page ([#218](https://github.com/mnemnk/mnemnk-app/issues/218)) ([0c81fab](https://github.com/mnemnk/mnemnk-app/commit/0c81fab22c64f508e95ffb69f0b4689c20a3cfd3))
* 🎸 separate init and ready process ([#222](https://github.com/mnemnk/mnemnk-app/issues/222)) ([142c2c3](https://github.com/mnemnk/mnemnk-app/commit/142c2c39b7b06199d8467ecbad0f10b9afa07c76))
* 🎸 start and stop only selected agents ([#219](https://github.com/mnemnk/mnemnk-app/issues/219)) ([f5bdffb](https://github.com/mnemnk/mnemnk-app/commit/f5bdffb84718f7e2222ff03b7ad7704d5b5093c0))


### Bug Fixes

* 🐛 agent can't be started after importing flow ([#214](https://github.com/mnemnk/mnemnk-app/issues/214)) ([a29951c](https://github.com/mnemnk/mnemnk-app/commit/a29951cd689ce81ab83a5f835f917946842c9802))
* 🐛 path is remaining even after importing flow ([#215](https://github.com/mnemnk/mnemnk-app/issues/215)) ([5eb5d34](https://github.com/mnemnk/mnemnk-app/commit/5eb5d342dae96970e139c4022b4cfc9bc1f97713))
* 🐛 play and pause don't work when no nodes are selected ([#223](https://github.com/mnemnk/mnemnk-app/issues/223)) ([5d1840a](https://github.com/mnemnk/mnemnk-app/commit/5d1840a276f7af515e75bd5faf22ce5f41cb3abe))

## [0.14.0](https://github.com/mnemnk/mnemnk-app/compare/v0.13.3...v0.14.0) (2025-04-08)


### ⚠ BREAKING CHANGES

* 🧨 Add ch to input and output
* 🧨 change text value in config from array to string

* 💡 change text value from array to string ([#206](https://github.com/mnemnk/mnemnk-app/issues/206)) ([3fd87b0](https://github.com/mnemnk/mnemnk-app/commit/3fd87b0288d277b3e1863e765bfba65eae0dd06c))


### Features

* 🎸 Add ch to input and output ([#213](https://github.com/mnemnk/mnemnk-app/issues/213)) ([3c283c7](https://github.com/mnemnk/mnemnk-app/commit/3c283c7132081b0f7ef6d4d10a2333f4a8b4b9d7))
* 🎸 As Kind agent ([#204](https://github.com/mnemnk/mnemnk-app/issues/204)) ([8dfb692](https://github.com/mnemnk/mnemnk-app/commit/8dfb692bde2256696fe272894a91042ddae8d5d4))
* 🎸 Display Messages agent ([#200](https://github.com/mnemnk/mnemnk-app/issues/200)) ([c63c0f0](https://github.com/mnemnk/mnemnk-app/commit/c63c0f021145bbe18617e213a3fd2da48e0cffa2))
* 🎸 Template String agent ([#205](https://github.com/mnemnk/mnemnk-app/issues/205)) ([d81054d](https://github.com/mnemnk/mnemnk-app/commit/d81054d07bc3d24a2d73d67aa954abdf7de9fb40))
* 🎸 Template Text agent ([#207](https://github.com/mnemnk/mnemnk-app/issues/207)) ([a9a63de](https://github.com/mnemnk/mnemnk-app/commit/a9a63de2e9db14096c356de10e4a121f4ceffaee))
* 🎸 Unit Input agent ([#208](https://github.com/mnemnk/mnemnk-app/issues/208)) ([a279261](https://github.com/mnemnk/mnemnk-app/commit/a2792614fff6da752a3e96640d7dfb622afe2a5f))
* 🎸 upadte config by enter ([#195](https://github.com/mnemnk/mnemnk-app/issues/195)) ([24428cd](https://github.com/mnemnk/mnemnk-app/commit/24428cd5b37bc62d305955acc7456ae49c5a5075))
* 🎸 update Display Messages agent ([#203](https://github.com/mnemnk/mnemnk-app/issues/203)) ([fc4ab8b](https://github.com/mnemnk/mnemnk-app/commit/fc4ab8b122c87bb401593c2c558b1462e7c93d7f))


### Bug Fixes

* 🐛 remove invalid/obsolete edges ([#202](https://github.com/mnemnk/mnemnk-app/issues/202)) ([044bdfb](https://github.com/mnemnk/mnemnk-app/commit/044bdfb26c61bc3d3c588bfcc0d3946ddaaea946))
* 🐛 remove unused import ([#198](https://github.com/mnemnk/mnemnk-app/issues/198)) ([4429d51](https://github.com/mnemnk/mnemnk-app/commit/4429d5146b654a1dec5e8812283cbb114663872b))

## [0.13.3](https://github.com/mnemnk/mnemnk-app/compare/v0.13.2...v0.13.3) (2025-04-04)


### Features

* 🎸 agent category ([#191](https://github.com/mnemnk/mnemnk-app/issues/191)) ([9a7444b](https://github.com/mnemnk/mnemnk-app/commit/9a7444b3382f56fda9db71466a946803973272b8))
* 🎸 agent list ([#190](https://github.com/mnemnk/mnemnk-app/issues/190)) ([77460ff](https://github.com/mnemnk/mnemnk-app/commit/77460ff6b55049d15030aac5c749b9c99b3fdc1e))
* 🎸 rename node title ([#192](https://github.com/mnemnk/mnemnk-app/issues/192)) ([9214a03](https://github.com/mnemnk/mnemnk-app/commit/9214a03825bc47ba1ed1f32cc813078d7de17a4f)), closes [#188](https://github.com/mnemnk/mnemnk-app/issues/188)
* 🎸 sort flow names in flow mega menu ([#193](https://github.com/mnemnk/mnemnk-app/issues/193)) ([4e21bee](https://github.com/mnemnk/mnemnk-app/commit/4e21beea228909c712a89b4254cf8f40598b7897))

## [0.13.2](https://github.com/mnemnk/mnemnk-app/compare/v0.13.1...v0.13.2) (2025-04-04)


### Features

* 🎸 config types ([#181](https://github.com/mnemnk/mnemnk-app/issues/181)) ([9c36181](https://github.com/mnemnk/mnemnk-app/commit/9c361816143826a710e7bf1bb274c1d276be9eab))
* 🎸 delete agent flow ([#187](https://github.com/mnemnk/mnemnk-app/issues/187)) ([e130c43](https://github.com/mnemnk/mnemnk-app/commit/e130c432f0200c05cd2dd399a662c866342896cb)), closes [#161](https://github.com/mnemnk/mnemnk-app/issues/161)
* 🎸 grow textarea on resizing nodes ([#189](https://github.com/mnemnk/mnemnk-app/issues/189)) ([82c300d](https://github.com/mnemnk/mnemnk-app/commit/82c300de32628122f9e3a0fd507dd5ace8124dee)), closes [#170](https://github.com/mnemnk/mnemnk-app/issues/170)
* 🎸 input agents ([#180](https://github.com/mnemnk/mnemnk-app/issues/180)) ([bb6b774](https://github.com/mnemnk/mnemnk-app/commit/bb6b774583eda0f6e97ab2ff6c597d03b8551c4a))


### Bug Fixes

* 🐛 disabled agents still work ([#182](https://github.com/mnemnk/mnemnk-app/issues/182)) ([a8cb918](https://github.com/mnemnk/mnemnk-app/commit/a8cb91855fb3f56bb34bd02b66f87d6fb1f63d73)), closes [#179](https://github.com/mnemnk/mnemnk-app/issues/179)
* 🐛 inconsistent state between nodes and flows ([#184](https://github.com/mnemnk/mnemnk-app/issues/184)) ([c603576](https://github.com/mnemnk/mnemnk-app/commit/c6035765df156b912a8fc33cb3f32e48d9337174)), closes [#183](https://github.com/mnemnk/mnemnk-app/issues/183)
* 🐛 re-enable MiniMap ([#186](https://github.com/mnemnk/mnemnk-app/issues/186)) ([6734a6e](https://github.com/mnemnk/mnemnk-app/commit/6734a6e3432aa1c5e69460b30d11997f81c8d2cf)), closes [#162](https://github.com/mnemnk/mnemnk-app/issues/162)

## [0.13.1](https://github.com/mnemnk/mnemnk-app/compare/v0.13.0...v0.13.1) (2025-04-03)


### Features

* 🎸 resize node ([#174](https://github.com/mnemnk/mnemnk-app/issues/174)) ([3741764](https://github.com/mnemnk/mnemnk-app/commit/3741764901c5d498c869932c70c573cc8961ba7a)), closes [#170](https://github.com/mnemnk/mnemnk-app/issues/170)

### Bug Fixes

* 🐛 display value does not show any value ([#178](https://github.com/mnemnk/mnemnk-app/issues/178)) ([00c6e3a](https://github.com/mnemnk/mnemnk-app/commit/00c6e3ad48075afe4fd1e71f4b842e8fbb70810c)), closes [#177](https://github.com/mnemnk/mnemnk-app/issues/177)
* 🐛 save flow ([#176](https://github.com/mnemnk/mnemnk-app/issues/176)) ([45f0273](https://github.com/mnemnk/mnemnk-app/commit/45f02735c190609438e45278394f1df52e3e8a11)), closes [#175](https://github.com/mnemnk/mnemnk-app/issues/175)

## [0.13.0](https://github.com/mnemnk/mnemnk-app/compare/v0.12.1...v0.13.0) (2025-04-03)


### ⚠ BREAKING CHANGES

* 🧨 configs are changed from record to list in mnemnk.json

### Features

* 🎸 display value on node ([#168](https://github.com/mnemnk/mnemnk-app/issues/168)) ([5375d5d](https://github.com/mnemnk/mnemnk-app/commit/5375d5d11fad96d206e11cac68383261080ed976))
* 🎸 regex filter ([#169](https://github.com/mnemnk/mnemnk-app/issues/169)) ([095b139](https://github.com/mnemnk/mnemnk-app/commit/095b1395272cc016c84655a3d36fa2eb9bff57ae))
* 🎸 remove JsonPath agent ([#171](https://github.com/mnemnk/mnemnk-app/issues/171)) ([23eeee9](https://github.com/mnemnk/mnemnk-app/commit/23eeee90aa25626338d66062a4ad05fc2543be28))
* 🎸 rename agent flow ([#163](https://github.com/mnemnk/mnemnk-app/issues/163)) ([d08a8d7](https://github.com/mnemnk/mnemnk-app/commit/d08a8d7e488b4211570f81f4ca6f6e5daa0884aa)), closes [#160](https://github.com/mnemnk/mnemnk-app/issues/160)
* 🎸 use vec for configs in agent def ([#173](https://github.com/mnemnk/mnemnk-app/issues/173)) ([1dc50cd](https://github.com/mnemnk/mnemnk-app/commit/1dc50cdd5aa37e2ecd97ec0d2915557c243abcf7))


### Bug Fixes

* 🐛 remove MiniMap for now ([#164](https://github.com/mnemnk/mnemnk-app/issues/164)) ([80c33fd](https://github.com/mnemnk/mnemnk-app/commit/80c33fd4b9bd97b25dac28a0e258ab9982883c1a)), closes [#162](https://github.com/mnemnk/mnemnk-app/issues/162)

## [0.12.1](https://github.com/mnemnk/mnemnk-app/compare/v0.12.0...v0.12.1) (2025-04-01)


### Features

* 🎸 add support default_config.type object ([#144](https://github.com/mnemnk/mnemnk-app/issues/144)) ([6326fae](https://github.com/mnemnk/mnemnk-app/commit/6326fae585653c4157c7ba60f4f0d8112b3015b2))
* 🎸 AgentMegaMenu ([#153](https://github.com/mnemnk/mnemnk-app/issues/153)) ([68ca2dd](https://github.com/mnemnk/mnemnk-app/commit/68ca2dd7a046b6ce91d3bf1b2d43c3d07620eae9))
* 🎸 global config ([#147](https://github.com/mnemnk/mnemnk-app/issues/147)) ([7645cb4](https://github.com/mnemnk/mnemnk-app/commit/7645cb4ecd253bca9ceda1a69e29e0dde483e8da))
* 🎸 migrate SvelteFlow to v1 ([#143](https://github.com/mnemnk/mnemnk-app/issues/143)) ([f41e3ed](https://github.com/mnemnk/mnemnk-app/commit/f41e3ed29fc44f61aafe6d96afad8eb3d0e37f7d))
* 🎸 new ids on importing ([#158](https://github.com/mnemnk/mnemnk-app/issues/158)) ([fc659ee](https://github.com/mnemnk/mnemnk-app/commit/fc659ee6aaeb1a12b5eb0eb9ddf187a328e26686)), closes [#148](https://github.com/mnemnk/mnemnk-app/issues/148)
* 🎸 play and pause buttons ([#159](https://github.com/mnemnk/mnemnk-app/issues/159)) ([96da69f](https://github.com/mnemnk/mnemnk-app/commit/96da69fb912d050aac91502034a19732be8a10cb))
* 🎸 realtime changes to config and enabled on frontend ([#157](https://github.com/mnemnk/mnemnk-app/issues/157)) ([fb0a2f5](https://github.com/mnemnk/mnemnk-app/commit/fb0a2f59462b3a7223cadbb7404d58536f1ccde5))


### Bug Fixes

* 🐛 boolean config and enabled ([#150](https://github.com/mnemnk/mnemnk-app/issues/150)) ([2b01110](https://github.com/mnemnk/mnemnk-app/commit/2b0111011380f6253699036f03e15afe4d97e6f5))
* 🐛 migrate SvelteFlow v1 ([#146](https://github.com/mnemnk/mnemnk-app/issues/146)) ([f4cca22](https://github.com/mnemnk/mnemnk-app/commit/f4cca22572f3f2d24ed1e53291d920e1d399b9b6))
* 🐛 update frontend flow after saving ([#156](https://github.com/mnemnk/mnemnk-app/issues/156)) ([632908c](https://github.com/mnemnk/mnemnk-app/commit/632908c940521eedaa637639e6349e2bb3b53f40))
* 🐛 use flow name on exporting and importing flow ([#155](https://github.com/mnemnk/mnemnk-app/issues/155)) ([4d3c8d2](https://github.com/mnemnk/mnemnk-app/commit/4d3c8d236c495bf53b89c6fe3dc25f920abe6d95))
* 🐛 use renamed name on duplicated flow name ([#154](https://github.com/mnemnk/mnemnk-app/issues/154)) ([e5632df](https://github.com/mnemnk/mnemnk-app/commit/e5632df6a92433fe0ee212c7a286ec7d35d3d8b4))

## [0.12.0](https://github.com/mnemnk/mnemnk-app/compare/v0.11.3...v0.12.0) (2025-03-30)


### ⚠ BREAKING CHANGES

* 🧨 protocol change. remove source arg from agent input

### Features

* 🎸 empty board name is invalid on Board In Agent ([#139](https://github.com/mnemnk/mnemnk-app/issues/139)) ([c818b89](https://github.com/mnemnk/mnemnk-app/commit/c818b89854616052b36a4377ba4f606a1b1c018b))
* 🎸 improve UI ([#142](https://github.com/mnemnk/mnemnk-app/issues/142)) ([083e90c](https://github.com/mnemnk/mnemnk-app/commit/083e90c2780fbcb6dc6d24b2e31b360b20e45afd))
* 🎸 JSON Path Agent ([#135](https://github.com/mnemnk/mnemnk-app/issues/135)) ([0ed2f9d](https://github.com/mnemnk/mnemnk-app/commit/0ed2f9da8c0e971be5aad64b33a14e66fdc3a47f))
* 🎸 mnemnk.json supports multiple agent definitions ([#136](https://github.com/mnemnk/mnemnk-app/issues/136)) ([a7f7896](https://github.com/mnemnk/mnemnk-app/commit/a7f78962d14d1010ed1c8b505c3748856623733c))
* 🎸 new entry command.args for command agent definition ([#133](https://github.com/mnemnk/mnemnk-app/issues/133)) ([933147b](https://github.com/mnemnk/mnemnk-app/commit/933147b43e5ab5091805e5b2ff91ae697fc683d6))
* 🎸 open and close agent drawer ([#140](https://github.com/mnemnk/mnemnk-app/issues/140)) ([c61dd26](https://github.com/mnemnk/mnemnk-app/commit/c61dd26f6ff90ae5409da4d7789fb4efc150d785))
* 🎸 remove source arg from agent input ([#134](https://github.com/mnemnk/mnemnk-app/issues/134)) ([2e7b306](https://github.com/mnemnk/mnemnk-app/commit/2e7b306493700b470f3c806726b97d8a03fd05c5))
* 🎸 separate board into in- and out- ([#138](https://github.com/mnemnk/mnemnk-app/issues/138)) ([5be505f](https://github.com/mnemnk/mnemnk-app/commit/5be505f89c3c8728eef217df618cf5fcb2f12cb7))
* 🎸 support multiple flows ([#141](https://github.com/mnemnk/mnemnk-app/issues/141)) ([a03fc97](https://github.com/mnemnk/mnemnk-app/commit/a03fc97ca778a6c3a385f9bad0db604ed7679731))
* 🎸 treat empty handle name as * ([#137](https://github.com/mnemnk/mnemnk-app/issues/137)) ([cc0430b](https://github.com/mnemnk/mnemnk-app/commit/cc0430bf1cc1e8e8c0acf7e576a799c7a68a22fa))


### Bug Fixes

* 🐛 change input(&self, into input(&mut self ([#132](https://github.com/mnemnk/mnemnk-app/issues/132)) ([7110d64](https://github.com/mnemnk/mnemnk-app/commit/7110d6439d2a4deaba0428f35712a770cf3f3ea6))
* 🐛 update data.config ([64c968d](https://github.com/mnemnk/mnemnk-app/commit/64c968d72b5ea64014be5810e79dccb42bcd2ba1))

## [0.11.3](https://github.com/mnemnk/mnemnk-app/compare/v0.11.2...v0.11.3) (2025-03-20)


### Features

* 🎸 export and import agent flow ([#118](https://github.com/mnemnk/mnemnk-app/issues/118)) ([9a8792f](https://github.com/mnemnk/mnemnk-app/commit/9a8792f1d712145fa49bb546340e287aa4377e80)), closes [#116](https://github.com/mnemnk/mnemnk-app/issues/116)
* 🎸 initial settings ([#119](https://github.com/mnemnk/mnemnk-app/issues/119)) ([cfd0d0a](https://github.com/mnemnk/mnemnk-app/commit/cfd0d0a7703102044105c06acbb5ffdb5d502c8c))
* 🎸 prohibit absolute path in agent config ([#120](https://github.com/mnemnk/mnemnk-app/issues/120)) ([bfaeef2](https://github.com/mnemnk/mnemnk-app/commit/bfaeef273fa6eaff4b63faf5d7ed4742981cc7f2))


### Bug Fixes

* 🐛 remove rebuild search index ([#121](https://github.com/mnemnk/mnemnk-app/issues/121)) ([e523f86](https://github.com/mnemnk/mnemnk-app/commit/e523f86fa14be1239672c9b530966a785807af31))

## [0.11.2](https://github.com/mnemnk/mnemnk-app/compare/v0.11.1...v0.11.2) (2025-03-19)


### Bug Fixes

* 🐛 save and load settings ([#117](https://github.com/mnemnk/mnemnk-app/issues/117)) ([0e60c95](https://github.com/mnemnk/mnemnk-app/commit/0e60c95cefaedba58d774433ca417497a9ea6d75))

## [0.11.1](https://github.com/mnemnk/mnemnk-app/compare/v0.11.0...v0.11.1) (2025-03-19)


### Bug Fixes

* 🐛 two store files exist ([#115](https://github.com/mnemnk/mnemnk-app/issues/115)) ([49d6e53](https://github.com/mnemnk/mnemnk-app/commit/49d6e532a2ce76d6f25499aff1af82fe6dddc1ba))

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
