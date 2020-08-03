# Huobi Chain

[![License](https://img.shields.io/badge/License-Apache%202.0-green.svg)](https://opensource.org/licenses/Apache-2.0)

简体中文 | [English](./README_EN.md)

## 简介

火币公链是 [火币集团](https://www.huobigroup.com/) 和 [Nervos Foundation](https://www.nervos.org/) 联合开发的高性能公链。它基于 [Muta](https://github.com/nervosnetwork/muta)、[CKB-VM](https://github.com/nervosnetwork/ckb-vm)、[Overlord](https://github.com/nervosnetwork/overlord)、[Nervos-p2p](https://github.com/nervosnetwork/p2p) 等开源组件深度定制，并面向金融应用的场景进行了扩展与优化。

火币公链目前的主要技术特征有：共识算法采用自适应流水线算法提高交易吞吐量；采用聚合签名技术降低共识算法延迟；使用基于账户的 CKB-VM 实现编译器和硬件友好的高性能智能合约虚拟机；内置一等资产类型，对用户资产采用内置服务管理，大幅提高安全性、通用性并降低复杂度；支持原生跨链协议，火币公链、[Nervos CKB](https://github.com/nervosnetwork/ckb) 和基于火币公链或 [Muta](https://github.com/nervosnetwork/muta) 技术开发的侧链可以直接实现跨链；支持高灵活性的 service 定制，来适应不同业务场景；支持合约开发，赋能去中心化应用。

在面向金融应用优化方面，火币公链支持或在未来计划支持交易确定性回执、单账号交易并发处理、任意资产支付交易手续费、第三方代付手续费、金融行业 DSL 执行环境等区别于大多数公链的特性。此外，火币公链还将提供可插拔的监管组件，根据应用场景可选择地对合约部署、运行，资产持有与转移，KYC 与 AML 等进行监管对接。

## 开始

- [快速入门](https://huobigroup.github.io/huobi-chain-docs/#/getting_started)
- [文档](https://huobigroup.github.io/huobi-chain-docs/#/)

## 核心功能

> 提示：火币公链目前正处在早期开发中，技术细节、设计文档和实现代码会频繁变更。

#### 共识和执行完全并行

火币公链采用 [Overlord][overlord] 共识算法，其设计目标是成为能够支持上百个共识节点，满足数千笔每秒的交易处理能力，且交易延迟不超过数秒的 BFT 共识算法。Overlord 的核心思想是解耦交易定序与状态共识，从而实现共识和执行完全并行，极大提高整条链的交易吞吐量。

#### 内置跨链能力

采用 FCA 实例化的 UDT 具备原生跨链功能，火币公链与其侧链，以及火币公链与 Nervos CKB 公链之间都可以采用这种跨链协议实现去中心化跨链。

我们采用去中心化 relay 的方式传递跨链证明，relayer 可以是侧链 validator，也可以是其他第三方用户。侧链之间可以不依赖火币公链或 Nervos CKB 实现直接的跨链功能。

#### 可灵活定制的 service

Service 是 Muta 框架中用于扩展的抽象层，用户可以基于 Service 定义区块治理、添加 VM、或实现一个 dapp。当前火币公链测试链基于 Muta 框架内置了四个 build-in service： asset service，risc-v service， metadata service， node manager service。未来火币公链将会通过 service 添加更多的功能特性，满足应用需求和监管需求。未来火币公链的侧链在实现高性能的特定业务时，也可复用这些 service。

#### 应用开发

用户在当前的测试网上可以发行自己的代币，也可以通过合约开发去中心化的应用。目前支持 C 语言编写合约，合约将被编译成 RISC-V 代码，动态部署到链上。后续 Huobi Chain 将会支持更多的合约编程语言，进一步完善开发者的体验。

## 贡献 ![PRs](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)

- :fork_and_knife:Fork 这个项目并 clone 到本地
- :twisted_rightwards_arrows:新建一个分支: `git checkout -b new-branch`
- :wrench:增加新特性或者解决一些 bug
- :memo:提交你的修改: `git commit -am 'Add some feature'`
- :rocket:推送你的分支: `git push origin new-branch`
- :tada:提交 Pull Request

或者提交一个[issue](https://github.com/HuobiGroup/huobi-chain/issues) - 欢迎任何有帮助性的建议:stuck_out_tongue_winking_eye:

如果愿意参与翻译文档，请到未翻译的英文文档页面上方点击 edit on GitHub 可以找到源文件，直接修改源文件，并且提 PR，步骤同上面的 PR 步骤。

[overlord]: https://github.com/cryptape/overlord
[risc-v]: https://www.wikiwand.com/en/RISC-V
[eip-150]: https://docs.google.com/spreadsheets/d/1n6mRqkBz3iWcOlRem_mO09GtSKEKrAsfO7Frgx18pNU/edit#gid=0
[ckb-vm]: https://github.com/nervosnetwork/ckb-vm
[minits]: https://github.com/cryptape/minits
[move]: https://developers.libra.org/docs/move-overview
[ckb-white-paper]: https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0002-ckb/0002-ckb.md
