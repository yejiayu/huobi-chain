# [](https://github.com/LycrusHamster/huobi-chain/compare/v0.5.0-beta.1...v) (2020-08-04)


### Bug Fixes

* fix kyc orgName and tagName length to 32, values up to 16 ([#108](https://github.com/LycrusHamster/huobi-chain/issues/108)) ([484c64d](https://github.com/LycrusHamster/huobi-chain/commit/484c64dee207a28bbb62335fe7466cb087b82eac))



# [0.5.0-beta.1](https://github.com/LycrusHamster/huobi-chain/compare/v0.5.0-alpha...v0.5.0-beta.1) (2020-08-03)


### Bug Fixes

* update genesis by using known private ([#33](https://github.com/LycrusHamster/huobi-chain/issues/33)) ([c67e526](https://github.com/LycrusHamster/huobi-chain/commit/c67e52639e1fa6b9b892f855f7f63a8ef67cd467))
* **governance:** emit only one charge fee event ([#28](https://github.com/LycrusHamster/huobi-chain/issues/28)) ([8ede4af](https://github.com/LycrusHamster/huobi-chain/commit/8ede4af08076cc509fa28e4cef926b293a1dc860))


### Features

* **service:** add query admin method for service ([#25](https://github.com/LycrusHamster/huobi-chain/issues/25)) ([e06c0c2](https://github.com/LycrusHamster/huobi-chain/commit/e06c0c213a3651181ecdc2935ce8e4ac5f825b81))
* asset, kyc, riscv, basic e2e tests ([#22](https://github.com/LycrusHamster/huobi-chain/issues/22)) ([c0f21ce](https://github.com/LycrusHamster/huobi-chain/commit/c0f21ce17719fbccc6daa27eadff95fedb040310))



# [0.5.0-alpha](https://github.com/LycrusHamster/huobi-chain/compare/v0.4.0-rc1...v0.5.0-alpha) (2020-07-15)


### Bug Fixes

* **gov:** clear fee at tx_hook_before ([#18](https://github.com/LycrusHamster/huobi-chain/issues/18)) ([bc464b0](https://github.com/LycrusHamster/huobi-chain/commit/bc464b0b6ecfff75820b015f59f639e497334096))
* package.json, lint ([#5](https://github.com/LycrusHamster/huobi-chain/issues/5)) ([c4f6e53](https://github.com/LycrusHamster/huobi-chain/commit/c4f6e5396f1804dde487141e1389e5ee1a923f5b))
* **admission_control:** clippy warnings ([3035a20](https://github.com/LycrusHamster/huobi-chain/commit/3035a20b41b5713f064d082e6183dd3de08717a7))
* **asset:** clippy warnings ([38f037e](https://github.com/LycrusHamster/huobi-chain/commit/38f037e4ef03003c32e280f3b77f7d022d2af6e2))
* **test:** e2e tests ([#2](https://github.com/LycrusHamster/huobi-chain/issues/2)) ([518c49d](https://github.com/LycrusHamster/huobi-chain/commit/518c49d435d254db1ab7e5c256e7b9c131a99b3a))
* e2e test ([#3](https://github.com/LycrusHamster/huobi-chain/issues/3)) ([4f27a2e](https://github.com/LycrusHamster/huobi-chain/commit/4f27a2ecc850c21cc17b7518b28110cc5d8f0707))
* **asset:** clippy warnings ([44a9e5a](https://github.com/LycrusHamster/huobi-chain/commit/44a9e5ac4ec382321936f94efe1c74f704e5b366))
* **genesis:** panic on governance genesis initialization ([#81](https://github.com/LycrusHamster/huobi-chain/issues/81)) ([d780266](https://github.com/LycrusHamster/huobi-chain/commit/d780266531566eb412ca5b95ab412d99606f57fa))
* **governance:** accumulated profits aren't reset after tx ([#78](https://github.com/LycrusHamster/huobi-chain/issues/78)) ([b13a7a6](https://github.com/LycrusHamster/huobi-chain/commit/b13a7a6fc35cf1c68b92e81dbad056ec03787ea1))
* **kyc:** fix bugs, add test cases ([#77](https://github.com/LycrusHamster/huobi-chain/issues/77)) ([84f2e28](https://github.com/LycrusHamster/huobi-chain/commit/84f2e286c683c8b311da77d321a0752f857e0698))
* **riscv:** call fn can change state ([#59](https://github.com/LycrusHamster/huobi-chain/issues/59)) ([960edaa](https://github.com/LycrusHamster/huobi-chain/commit/960edaa5bb53b368c042f09e1117d4eb4e403b24))
* **riscv:** failed contract execution doesn't increase cycles ([#71](https://github.com/LycrusHamster/huobi-chain/issues/71)) ([f8f2eff](https://github.com/LycrusHamster/huobi-chain/commit/f8f2eff256fd40eff8713e45fbcbf67a97a99c89))
* **riscv:** pvm assert don't return assert message ([#72](https://github.com/LycrusHamster/huobi-chain/issues/72)) ([ef7c21e](https://github.com/LycrusHamster/huobi-chain/commit/ef7c21e745f9e5928cc89bc6ec0e64734cf7341f))


### Features

* **admission control:** check tx sender balance against failure fee ([#80](https://github.com/LycrusHamster/huobi-chain/issues/80)) ([12622bb](https://github.com/LycrusHamster/huobi-chain/commit/12622bb55659f0a67bb52ba0cc4b08a615eb023a))
* **asset:** add mint and burn ([02c3c2b](https://github.com/LycrusHamster/huobi-chain/commit/02c3c2b8d8c8cf841efe4d8f39ac76de361c2edf))
* **asset:** multi issuer support in genesis ([#14](https://github.com/LycrusHamster/huobi-chain/issues/14)) ([6ac89de](https://github.com/LycrusHamster/huobi-chain/commit/6ac89de3bc8aeec94d89c2cce4653f30e53b8fa6))
* **asset:** multi issuer support in genesis ([#14](https://github.com/LycrusHamster/huobi-chain/issues/14)) ([#17](https://github.com/LycrusHamster/huobi-chain/issues/17)) ([94929b2](https://github.com/LycrusHamster/huobi-chain/commit/94929b2b368aad8ae62b7f895e86113787329f01))
* **auth:** register admission control in authorization service ([#76](https://github.com/LycrusHamster/huobi-chain/issues/76)) ([a94739e](https://github.com/LycrusHamster/huobi-chain/commit/a94739ed3fdd8a40b2b445b0e883c2a366bd1ed3))
* **govern:** add governance service ([#69](https://github.com/LycrusHamster/huobi-chain/issues/69)) ([2e4e16b](https://github.com/LycrusHamster/huobi-chain/commit/2e4e16bc7adf74b8ff2a47ccf47d8ccb329adae1))
* **main:** register kyc and multi-signature service in main function ([#4](https://github.com/LycrusHamster/huobi-chain/issues/4)) ([22b9124](https://github.com/LycrusHamster/huobi-chain/commit/22b91245b78235c73bf889a681dd79e348574322))
* **riscv:** contract authorization ([#62](https://github.com/LycrusHamster/huobi-chain/issues/62)) ([2c6e836](https://github.com/LycrusHamster/huobi-chain/commit/2c6e8364951c7435a138bb158348859fba65bac8))
* **service:** add admission control services ([960037a](https://github.com/LycrusHamster/huobi-chain/commit/960037a6dcd74392995f82e68447dc3638465f40))


### Reverts

* Revert "feat(asset): multi issuer support in genesis (#14)" (#16) ([cfecf83](https://github.com/LycrusHamster/huobi-chain/commit/cfecf83a3b45470fe91068ed5b300bef96701926)), closes [#14](https://github.com/LycrusHamster/huobi-chain/issues/14) [#16](https://github.com/LycrusHamster/huobi-chain/issues/16)


* change!(asset): allow transfer to self ([e93d769](https://github.com/LycrusHamster/huobi-chain/commit/e93d7695f243167e96839b7aaedf5c03d6a001d3))


### BREAKING CHANGES

* **riscv:** - get_contract api now also return authorizer
- rename enable_whitelist to enable_authorization in genesis.toml
- rename whitelist to deploy_auth in genesis.toml

* change(riscv): rename approve_contract and revoke_contract

Use plural form

* fix(e2e): riscv test

* fix(riscv): clippy warnings

* style(riscv): rename mut_kind ot kind_mut

* style(riscv): aggregate check into macros

* style(riscv): code format

* feat(risv): emit event on grant/revoke contract and deploy

* fix(riscv): clippy warnings
* - remove TransferToSelf service error
* **riscv:** - change Addresses to AuthPayload
- change Addresses to AuthorizedList

* change(riscv): upgrade ckb-vm to 0.19.1
* **riscv:** - change EcallError to InvalidEcall

* test(riscv): readonly call fn

* fix(riscv): e2e test code after update ckv-vm

* fix(riscv): clippy warnings



# [0.4.0-rc1](https://github.com/LycrusHamster/huobi-chain/compare/v0.3.0...v0.4.0-rc1) (2020-06-04)


### Code Refactoring

* upgrade to muta v0.1.2-beta2 ([#57](https://github.com/LycrusHamster/huobi-chain/issues/57)) ([711d600](https://github.com/LycrusHamster/huobi-chain/commit/711d600287a3dfd6dd17a752b1054b816a184e2d))


### BREAKING CHANGES

* - Return ServiceResponse instead ProtocolResult

* refactor(metadata): upgrade to muta v0.1.2-beta1
* - Return ServiceResponse instead of ProtocolResult

* refactor(node_manager): upgrade to muta v0.1.2-beta1
* - Return ServiceResponse instead of ProtocolResult

* refactor(riscv): upgrade to muta v0.1.2-beta1
* - Return ServiceResponse instead of ProtocolResult

* chore(main): upgrade to muta v0.1.2-beta1

* fix(riscv): test code

* refactor(asset): code format

* fix(e2e): node manager test

* chore: bump riscv and metadata version

* fix(asset): temporarily ignore fee charge result

* fix(e2e): asset test

Disable fee check

* fix(riscv): mock dispatcher doesn't encode result to json

* change(riscv): riscv service errors

* fix(e2e): riscv test

* fix(e2e): basic test

* fix(e2e): change validators

* refactor(e2e): test code, add eslint

* chore: bump version to 0.4.0-beta2



# [0.3.0](https://github.com/LycrusHamster/huobi-chain/compare/v0.2.0...v0.3.0) (2020-04-10)


### Bug Fixes

* call asset service transfer_from by contract ([#52](https://github.com/LycrusHamster/huobi-chain/issues/52)) ([346a594](https://github.com/LycrusHamster/huobi-chain/commit/346a594cf15226bc0136f006b80826e78abca2c9))
* riscv contract call result serialized to json twice ([#55](https://github.com/LycrusHamster/huobi-chain/issues/55)) ([09aed26](https://github.com/LycrusHamster/huobi-chain/commit/09aed268cbc85482854081836112ebfcd5a5b791))



# [0.2.0](https://github.com/LycrusHamster/huobi-chain/compare/v0.1.0...v0.2.0) (2020-03-15)


### Features

* add riscv service write/read function ([#42](https://github.com/LycrusHamster/huobi-chain/issues/42)) ([6785053](https://github.com/LycrusHamster/huobi-chain/commit/67850534d7ba148467811d581542111c87dcf0cc))
* implement fixed tx fee model ([#43](https://github.com/LycrusHamster/huobi-chain/issues/43)) ([4ab6100](https://github.com/LycrusHamster/huobi-chain/commit/4ab61005814a65005447e64f22eb7baf0959b6e5))



# [0.1.0](https://github.com/LycrusHamster/huobi-chain/compare/v0.1.0-rc.2...v0.1.0) (2020-02-28)



# [0.1.0-rc.2](https://github.com/LycrusHamster/huobi-chain/compare/v0.1.0-rc.1...v0.1.0-rc.2) (2020-02-24)


### Bug Fixes

* riscv invalid contract cause panic problem ([#34](https://github.com/LycrusHamster/huobi-chain/issues/34)) ([7d4d671](https://github.com/LycrusHamster/huobi-chain/commit/7d4d6718dc47f6eb52002727dd9461a88f36a675))


### Features

* add whitelist feature for riscv ([#36](https://github.com/LycrusHamster/huobi-chain/issues/36)) ([f86b792](https://github.com/LycrusHamster/huobi-chain/commit/f86b792126523bf17aa11ab79624deb4e719a9d7))
* update muta ([#38](https://github.com/LycrusHamster/huobi-chain/issues/38)) ([300475d](https://github.com/LycrusHamster/huobi-chain/commit/300475d96b457ff358569bff9be62df4b0418a3a))



# [0.1.0-rc.1](https://github.com/LycrusHamster/huobi-chain/compare/v0.0.1...v0.1.0-rc.1) (2020-02-15)


### Features

* add end to end test ([#28](https://github.com/LycrusHamster/huobi-chain/issues/28)) ([f6e40be](https://github.com/LycrusHamster/huobi-chain/commit/f6e40be513d226ad71b86379746faa49851b5d6c))
* add get_contract method to riscv service ([#21](https://github.com/LycrusHamster/huobi-chain/issues/21)) ([28e3b22](https://github.com/LycrusHamster/huobi-chain/commit/28e3b22864961401094c8299c67e5cbd8fdbd25a))
* add send tx tutorial in getting_started doc ([#12](https://github.com/LycrusHamster/huobi-chain/issues/12)) ([1c69a9f](https://github.com/LycrusHamster/huobi-chain/commit/1c69a9fb03a29cdfc291fab62175a377828f7f63))



## [0.0.1](https://github.com/LycrusHamster/huobi-chain/compare/935ef7d6d3a14b292f0da85bbd3296040ec61221...v0.0.1) (2019-10-31)


### Bug Fixes

* rpc pull txs. ([4f9c27c](https://github.com/LycrusHamster/huobi-chain/commit/4f9c27c346babf2d5b7c802462be252ec9683742))
* **consensus:** encode overlord message and verify signature ([#39](https://github.com/LycrusHamster/huobi-chain/issues/39)) ([b7dbb44](https://github.com/LycrusHamster/huobi-chain/commit/b7dbb444682ad72b4a90c5564747203e2ba44e87))
* **consensus:** Get authority list returns none. ([#4](https://github.com/LycrusHamster/huobi-chain/issues/4)) ([ddeaf65](https://github.com/LycrusHamster/huobi-chain/commit/ddeaf65d862d27c039d8af75c04555e504240388))
* **mempool:** Always get the latest epoch id when `package`. ([#30](https://github.com/LycrusHamster/huobi-chain/issues/30)) ([76146e6](https://github.com/LycrusHamster/huobi-chain/commit/76146e61981ab47e9fffd3f4557fcfccb2b65a7f))
* **mempool:** broadcast new transactions ([#32](https://github.com/LycrusHamster/huobi-chain/issues/32)) ([6362425](https://github.com/LycrusHamster/huobi-chain/commit/63624253763efa1e2669a3f586c05693d4927a56))
* **mempool:** Fix concurrent insert bug of mempool ([#19](https://github.com/LycrusHamster/huobi-chain/issues/19)) ([1209134](https://github.com/LycrusHamster/huobi-chain/commit/120913493c758e0a9beb9672cd317c9eca48ef1b))
* **mempool:** Resize the queue to ensure correct switching. ([#18](https://github.com/LycrusHamster/huobi-chain/issues/18)) ([bd8fabc](https://github.com/LycrusHamster/huobi-chain/commit/bd8fabc61bd84eaaef0db9526d4548d1f512c798))
* **network:** dead lock in peer manager ([#24](https://github.com/LycrusHamster/huobi-chain/issues/24)) ([a90b995](https://github.com/LycrusHamster/huobi-chain/commit/a90b995fe2f392e23d30a5b1e0e488752eae92cd))
* **network:** fail to bootstrap if bootstrap isn't start already ([#46](https://github.com/LycrusHamster/huobi-chain/issues/46)) ([bd2a3f1](https://github.com/LycrusHamster/huobi-chain/commit/bd2a3f15079dbd4aec120e196eae7b52d72aeaa5))
* **network:** never reconnect bootstrap again after failure ([#22](https://github.com/LycrusHamster/huobi-chain/issues/22)) ([818abde](https://github.com/LycrusHamster/huobi-chain/commit/818abde3e60c970aacefffb7e1976b7c18a9302e))
* Ignore bootstraps when empty. ([#41](https://github.com/LycrusHamster/huobi-chain/issues/41)) ([6edc43b](https://github.com/LycrusHamster/huobi-chain/commit/6edc43b6299682a9b02337712b9008b5fcc4ecfd))
* **network:** NoSessionId Error ([#33](https://github.com/LycrusHamster/huobi-chain/issues/33)) ([b6466b6](https://github.com/LycrusHamster/huobi-chain/commit/b6466b6508df2d8f77fe421daaccc0a087890367))


### Features

* **api:** make API more user-friendly ([#38](https://github.com/LycrusHamster/huobi-chain/issues/38)) ([9c4b6f1](https://github.com/LycrusHamster/huobi-chain/commit/9c4b6f168afbc06f957c01d36b18289a3a825e61))
* **mempool:** implement cached batch txs broadcast ([#20](https://github.com/LycrusHamster/huobi-chain/issues/20)) ([b8f3c93](https://github.com/LycrusHamster/huobi-chain/commit/b8f3c93fb52166c61835b5c0e24b20deab56729d))
* **sync:** synchronization epoch ([#9](https://github.com/LycrusHamster/huobi-chain/issues/9)) ([ec52fe4](https://github.com/LycrusHamster/huobi-chain/commit/ec52fe460876a779c5dd085d9864c43c63fcda0f)), closes [#17](https://github.com/LycrusHamster/huobi-chain/issues/17) [#18](https://github.com/LycrusHamster/huobi-chain/issues/18)
* add compile and run in README ([#11](https://github.com/LycrusHamster/huobi-chain/issues/11)) ([bf88cd3](https://github.com/LycrusHamster/huobi-chain/commit/bf88cd34227779b0f5606acff0c1361663a3e036))
* add docker ([#31](https://github.com/LycrusHamster/huobi-chain/issues/31)) ([bbcabf0](https://github.com/LycrusHamster/huobi-chain/commit/bbcabf0729358852b2893a7ac8616baff491077e))
* change rlp in executor to fixed-codec ([#29](https://github.com/LycrusHamster/huobi-chain/issues/29)) ([faecce5](https://github.com/LycrusHamster/huobi-chain/commit/faecce53155ba80880e5660d968ecb4d9d91f6f3))
* Get balance. ([#28](https://github.com/LycrusHamster/huobi-chain/issues/28)) ([1c4643a](https://github.com/LycrusHamster/huobi-chain/commit/1c4643a9a30db3929989cd272fa7f252f93af3c4))
* **codec:** Add codec tests and benchmarks ([#22](https://github.com/LycrusHamster/huobi-chain/issues/22)) ([d157f29](https://github.com/LycrusHamster/huobi-chain/commit/d157f29e6c32ed0f1b82d7f95465e9ed5409d009))
* **consensus:** develop consensus interfaces ([#21](https://github.com/LycrusHamster/huobi-chain/issues/21)) ([67acb33](https://github.com/LycrusHamster/huobi-chain/commit/67acb330507e808fdfb670bf92eb7a69572a7c70))
* **consensus:** develop consensus provider and engine ([#28](https://github.com/LycrusHamster/huobi-chain/issues/28)) ([f8b0181](https://github.com/LycrusHamster/huobi-chain/commit/f8b0181d73ef72b4300166754fc56b5615022d5e))
* **consensus:** Execute the transactions on commit. ([#7](https://github.com/LycrusHamster/huobi-chain/issues/7)) ([cc8c93b](https://github.com/LycrusHamster/huobi-chain/commit/cc8c93bb418a12d21f099e6fdc8b00b2edf74bd6))
* **consensus:** joint overlord and chain ([#32](https://github.com/LycrusHamster/huobi-chain/issues/32)) ([ca828cb](https://github.com/LycrusHamster/huobi-chain/commit/ca828cbb78fea507ddd6b710fcf5aca7e1d4b38c))
* **consensus:** mutex lock and timer config ([#45](https://github.com/LycrusHamster/huobi-chain/issues/45)) ([87caf51](https://github.com/LycrusHamster/huobi-chain/commit/87caf51f9c3c8e1f46869ddc008eabed72e35ed2))
* **consensus:** Support trsanction executor. ([#6](https://github.com/LycrusHamster/huobi-chain/issues/6)) ([39561e3](https://github.com/LycrusHamster/huobi-chain/commit/39561e3bdb9e64b38eef88ed8a7e1313b4d2ae53))
* **executor:** Create genesis. ([#1](https://github.com/LycrusHamster/huobi-chain/issues/1)) ([e2d4bd9](https://github.com/LycrusHamster/huobi-chain/commit/e2d4bd95e12257b063dc873010a3eeb1bf1b4e5c))
* **graphql:** Support transfer and contract deployment ([#44](https://github.com/LycrusHamster/huobi-chain/issues/44)) ([44547e4](https://github.com/LycrusHamster/huobi-chain/commit/44547e4c87e071c32e25738fd6b4ef2603e109ce))
* **mempool:** fix fixed_codec ([#25](https://github.com/LycrusHamster/huobi-chain/issues/25)) ([5aedfaa](https://github.com/LycrusHamster/huobi-chain/commit/5aedfaadc5a58973c5a09afec03963e6a98b86b7))
* **mempool:** Remove cycle_limit ([#23](https://github.com/LycrusHamster/huobi-chain/issues/23)) ([0c57217](https://github.com/LycrusHamster/huobi-chain/commit/0c572172181ca445611aee6dfb593a587187facf))
* **native-contract:** Support for asset creation and transfer. ([#37](https://github.com/LycrusHamster/huobi-chain/issues/37)) ([2bffd25](https://github.com/LycrusHamster/huobi-chain/commit/2bffd25be4007cf0250ffcbdcbc9d57d50ae6b1d))
* **network:** log connected peer ips ([#23](https://github.com/LycrusHamster/huobi-chain/issues/23)) ([472c863](https://github.com/LycrusHamster/huobi-chain/commit/472c863f0fc0efae6716c7be742be833923bedcd))
* develop merkle root ([#17](https://github.com/LycrusHamster/huobi-chain/issues/17)) ([3e16726](https://github.com/LycrusHamster/huobi-chain/commit/3e167268323be81d87c758b6238664662ee0d35e))
* Fill in the main function ([#36](https://github.com/LycrusHamster/huobi-chain/issues/36)) ([c72fa73](https://github.com/LycrusHamster/huobi-chain/commit/c72fa73052f51dcf75b71b6058baadf2be768d27))
* **mempool:** Develop mempool's tests and benches  ([#9](https://github.com/LycrusHamster/huobi-chain/issues/9)) ([784b8e3](https://github.com/LycrusHamster/huobi-chain/commit/784b8e349f36e3546118d11b674fbf419c9cc4ad))
* **mempool:** Implement MemPool interfaces ([#8](https://github.com/LycrusHamster/huobi-chain/issues/8)) ([cfa68bf](https://github.com/LycrusHamster/huobi-chain/commit/cfa68bf7a0ef04b5e60ab4dbdb84b9a067e7bd00))
* **native_contract:** Add an adapter that provides access to the world state. ([#27](https://github.com/LycrusHamster/huobi-chain/issues/27)) ([39b8a80](https://github.com/LycrusHamster/huobi-chain/commit/39b8a80cfd2cd83dc5578a4c644893c762ded16d))
* **protocol:** Add the mempool traits ([#7](https://github.com/LycrusHamster/huobi-chain/issues/7)) ([7351281](https://github.com/LycrusHamster/huobi-chain/commit/735128173262fb1c3974579c13190fd794b9ef59))
* **protocol:** Add the underlying data structure. ([#5](https://github.com/LycrusHamster/huobi-chain/issues/5)) ([935ef7d](https://github.com/LycrusHamster/huobi-chain/commit/935ef7d6d3a14b292f0da85bbd3296040ec61221))
* **protocol:** Protobuf serialize ([#6](https://github.com/LycrusHamster/huobi-chain/issues/6)) ([a08d4ed](https://github.com/LycrusHamster/huobi-chain/commit/a08d4ed2b0fc710dc96ea452e62353c246b20cd0))
* **storage:** add storage test ([#18](https://github.com/LycrusHamster/huobi-chain/issues/18)) ([c1b559a](https://github.com/LycrusHamster/huobi-chain/commit/c1b559a78fd5d18feb0191bd2c82a689b63ff8ef))
* **storage:** Implement memory adapter API ([#11](https://github.com/LycrusHamster/huobi-chain/issues/11)) ([10e8709](https://github.com/LycrusHamster/huobi-chain/commit/10e8709ea989d58272ae65d595ce93d95117e768))
* **storage:** Implement storage ([#17](https://github.com/LycrusHamster/huobi-chain/issues/17)) ([d256faf](https://github.com/LycrusHamster/huobi-chain/commit/d256fafade6242438165f3b82479a6d1f6a2c721))
* **types:** Add account structure. ([#24](https://github.com/LycrusHamster/huobi-chain/issues/24)) ([7183c39](https://github.com/LycrusHamster/huobi-chain/commit/7183c39b810ee1992465dc0991a2c669e30f7fe8))



