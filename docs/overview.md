# 概览

- [概览](#%e6%a6%82%e8%a7%88)
  - [介绍](#%e4%bb%8b%e7%bb%8d)
  - [什么是区块链框架](#%e4%bb%80%e4%b9%88%e6%98%af%e5%8c%ba%e5%9d%97%e9%93%be%e6%a1%86%e6%9e%b6)
  - [Muta 的特点](#muta-%e7%9a%84%e7%89%b9%e7%82%b9)
    - [高性能](#%e9%ab%98%e6%80%a7%e8%83%bd)
    - [高吞吐量的共识算法](#%e9%ab%98%e5%90%9e%e5%90%90%e9%87%8f%e7%9a%84%e5%85%b1%e8%af%86%e7%ae%97%e6%b3%95)
    - [CKB-VM 上的 Account 编程模型](#ckb-vm-%e4%b8%8a%e7%9a%84-account-%e7%bc%96%e7%a8%8b%e6%a8%a1%e5%9e%8b)
    - [First-class Asset](#first-class-asset)

## 介绍

Muta 是一个由 Rust 编写的具备可扩展性的高性能区块链框架。它允许你使用 Rust 或 Typescript 编写你的业务逻辑，构建你的专有区块链。

同时，Muta 还是 Nervos layer2 解决方案 Axon 的底层基础设施，Muta 将内置一套跨链方案联通整个 Nervos 网络。

## 什么是区块链框架

有过互联网开发经验的或多或少都会使用一些 Web 框架（eg. Express， Ruby on Rails），这些框架融合了 HTTP 协议解析、URL 解析、请求路由、中间件等必要功能，使用者只需要专注于每个 API 对应的业务逻辑即可。

区块链框架也一样，在区块链中 **智能合约+链上治理** 即为区块链中的业务逻辑，框架在底层提供了区块链运行的必要模块（eg. 共识算法、虚拟机、P2P）并且提供了稳定性、高性能的保证。

## Muta 的特点

### 高性能

Muta 的目标是达到每秒处理数千个事务（TPS）。在目前的 benchmark 环境中，Muta 的每秒处理事务（TPS）数大约在 2000 左右，出块间隔在 2.5s。

### 高吞吐量的共识算法

[Overlord][overlord] 是由 Nervos 研究团队设计研发的 BFT 类共识算法，其设计目标是成为能够支持上百个共识节点，满足数千笔每秒的交易处理能力，且交易延迟不超过数秒。Overlord 的核心思想是解耦交易定序与状态共识，从而实现共识和执行完全并行，极大提高整条链的交易吞吐量。

### CKB-VM 上的 Account 编程模型

[CKB-VM][ckb-vm] 是一个实现了 RISCV 指令集的区块链虚拟机，具有高性能，可扩展性，灵活性等优点。

在 Muta 中，智能合约的编程模型采用的是 Account 模型，相比 UTXO，Account 模型更便于开发复杂逻辑的智能合约。 Account 模型最早是由以太坊所采用的一种智能合约编程模型，并且许多关于操作 Account 的指令都内嵌到了以太坊虚拟机 EVM 中，

得益于 [CKB-VM][ckb-vm] 的灵活性和可扩展性，在不侵入指令集修改的前提下，我们在 [CKB-VM][ckb-vm] 之上实现了一套 Account SDK 以实现 Muta 中的 Account 模型， 不仅如此，我们还提供了合约编程语言 [Minits][minits]， [Minits][minits] 是一个专为区块链智能合约开发设计的 Typescript 的子集，它使用 LLVM 最终把代码编译成 RISCV 在 [CKB-VM][ckb-vm] 中运行。

### First-class Asset

在以太坊等智能合约平台中，用户自定义代币（User Defined Token, UDT）通常以标准智能合约形式出现。平台对某合约记录的是资产还是普通数据没有区分。这样带来了安全性、通用性和复杂性等多重风险。

Libra 使用的 Move 以及 Nervos CKB 提出了区块链中一等公民（first-class citizen）的概念。在这个概念中资产，或者所属权成为了系统可直接识别的数据，而不是和其他数据糅合在一起对系统保持透明。

Muta 对原生代币和 UDT 设置了一等公民的地位，我们称之为 first-class asset（简称 FCA）。所有代币的基础行为均由系统提供的原生合约实现，用户只需要给出代币的名称、发行量、管理方式等定义即可创建一等资产。

这样做的优势除了在于大幅降低实现复杂度、统一资产标准、提高安全性之外，更重要的是系统对代币行为可感知，从而便于实现更底层的经济激励、手续费计算逻辑以及原生跨链等业务。

[关于 First-class Asset 的更多讨论][first-class asset]

[overlord]: https://github.com/cryptape/overlord
[ckb-vm]: https://github.com/nervosnetwork/ckb-vm
[minits]: https://github.com/cryptape/minits
[first-class asset]: https://talk.nervos.org/t/first-class-asset/405
