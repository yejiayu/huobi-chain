# Huobi Chain
[![License](https://img.shields.io/badge/License-Apache%202.0-green.svg)](https://opensource.org/licenses/Apache-2.0)

简体中文 | [English](./README_EN.md)

## Introduction

火币公链是 [火币集团](https://www.huobigroup.com/) 和 [Nervos Foundation](https://www.nervos.org/) 联合开发的高性能公链。它基于 Muta、CKB-VM、Overlord、nervos-p2p 等开源组件深度定制，并面向金融应用的场景进行了扩展与优化。

火币公链目前的主要技术特征有：共识算法采用自适应流水线算法提高交易吞吐量；采用聚合签名技术降低共识算法延迟；使用基于账户的 CKB-VM 实现编译器和硬件友好的高性能智能合约虚拟机；内置一等资产类型，对用户资产采用内置系统合约管理，大幅提高安全性、通用性并降低复杂度；支持原生跨链协议，火币公链、Nervos CKB 和基于火币公链或 Muta 技术开发的侧链可以直接实现跨链；支持高灵活性的虚拟机合约以及高性能的原生合约，适应不同业务场景。

在面向金融应用优化方面，火币公链支持或在未来计划支持交易确定性回执、单账号交易并发处理、任意资产支付交易手续费、第三方代付手续费、金融行业 DSL 执行环境等区别于大多数公链的特性。此外，火币公链还将提供可插拔的监管组件，根据应用场景可选择地对合约部署、运行，资产持有与转移，KYC 与 AML 等进行监管对接。

## Get Started

- [Install](./docs/getting_started.md)
- [Documents](./docs/index.md)

## Key Features

> 提示：火币公链目前正处在早期开发中，技术细节、设计文档和实现代码会频繁变更。

### Parallel Execution Consensus

火币公链采用 [Overlord][overlord] 共识算法，其设计目标是成为能够支持上百个共识节点，满足数千笔每秒的交易处理能力，且交易延迟不超过数秒的 BFT 共识算法。Overlord 的核心思想是解耦交易定序与状态共识，从而实现共识和执行完全并行，极大提高整条链的交易吞吐量。

### RISC-V Based Virtual Machine

火币公链默认的虚拟机是采用了 [RISC-V][risc-v] 指令集的 CKB-VM。RISC-V 是一套在 BSD 开源协议下分发的针对硬件的精简指令集。相对于区块链中常用的 EVM 和 WASM，CKB-VM 的性能更高，指令集稳定无需频繁硬分叉升级，以及有众多开源生态支持等优势。

得益于 [CKB-VM][ckb-vm] 的灵活性和可扩展性，在不侵入指令集修改的前提下，我们在 CKB-VM 之上实现了一套 Account SDK 以实现火币公链智能合约中的 Account 模型，不仅如此，我们还提供了合约编程语言 [Minits][minits]，Minits 是一个专为区块链智能合约开发设计的 Typescript 的子集，它使用 LLVM 最终把代码编译成 RISC-V binary 在 CKB-VM 中运行。


### Built-in Interoperation Capability

采用 FCA 实例化的 UDT 具备原生跨链功能，火币公链与其侧链，以及火币公链与 Nervos CKB 公链之间都可以采用这种跨链协议实现去中心化跨链。

我们采用去中心化 relay 的方式传递跨链证明，relayer 可以是侧链 validator，也可以是其他第三方用户。侧链之间可以不依赖火币公链或 Nervos CKB 实现直接的跨链功能。

### Native Services

用户可以采用两种方式部署智能合约，第一种是将合约编译成 RISC-V 代码，动态部署到链上；第二种是采用 Rust 语言实现本机代码部署原生合约。原生合约绕开虚拟机的解释执行过程，直接访问系统资源，具有更高效的性能。

未来火币公链的侧链可能广泛采用原生合约来实现高性能的特定业务。


[overlord]: https://github.com/cryptape/overlord
[risc-v]: https://www.wikiwand.com/en/RISC-V
[eip-150]: https://docs.google.com/spreadsheets/d/1n6mRqkBz3iWcOlRem_mO09GtSKEKrAsfO7Frgx18pNU/edit#gid=0
[ckb-vm]: https://github.com/nervosnetwork/ckb-vm
[minits]: https://github.com/cryptape/minits
[move]: https://developers.libra.org/docs/move-overview
[ckb-white-paper]: https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0002-ckb/0002-ckb.md
