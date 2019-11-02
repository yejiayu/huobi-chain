# Huobi Chain
[![License](https://img.shields.io/badge/License-Apache%202.0-green.svg)](https://opensource.org/licenses/Apache-2.0)

## Introduction

Huobi Chain is a high-performance public chain jointly developed by [Huobi] (https://www.huobigroup.com/) and [Nervos Foundation] (https://www.nervos.org/). It is based on deep customization of open source components such as Muta, CKB-VM, Overlord, and nervos-p2p, and is extended and optimized for financial application scenarios.

The main technical features of the Huobi chain are: use adaptive pipeline consensus algorithm to improve transaction throughput; use aggregate signature technology to reduce the delay of consensus process; use account-based CKB-VM to implement compiler and hardware-friendly high-performance Virtual machine that support smart contract; have built-in first-class asset type, which means built-in system contract can be used to manager user assets, which greatly improving security, versatility, and complexity; support native cross-chain protocol which means Huobi public chain, Nervos CKB and other chain developed basing on Huobi chain or  Muta can be directly cross-chain communicated with each other; support for highly flexible virtual machine and high-performance native contracts to accommodate different business scenarios.

In terms of optimization for financial applications, Huobi chain plans to support transaction deterministic receipts, single account transaction concurrent processing, arbitrary asset payment for transaction fees, third party payment fees, financial industry DSL execution environment, etc. Most of the characteristics of Huobi chain. Besides, the Huobi Chain will provide pluggable regulatory components which can selectively supervise contract deployment, asset holding and transfer, KYC and AML depending on the application scenario.

## Get Started

- [Install](./docs/getting_started.md)
- [Documents](./docs/index.md)

## Key Features

> Huobi chain is currently in the early stages of development, so the technical details, design documentation, and implementation code are subject to frequent changes.

### Parallel Execution Consensus

The Huobi chain adopts the [Overlord][overlord] consensus algorithm, which is designed to be a BFT consensus algorithm capable of supporting hundreds of consensus nodes, satisfying thousands of transactions per second with trading delays of no more than a few seconds. The core idea of ​​Overlord is to decouple transaction sequencing and state consensus, so that the consensus module and the execution module can be executed in parallel, which greatly improves the transaction throughput of the entire chain.

### RISC-V Based Virtual Machine

The default virtual machine for the Huobi public chain is CKB-VM with the [RISC-V][risc-v] instruction set. RISC-V is an open-source hardware instruction set architecture (ISA) based on established reduced instruction set computer (RISC) principles. Compared with the commonly used EVM and WASM in the blockchain, CKB-VM has many advantages, including higher performance, stable instruction set without frequent hard fork upgrade, and support from open source ecology.

Thanks to the flexibility and extensibility of [CKB-VM][ckb-vm], we implemented an Account SDK on CKB-VM for the account model of Huobi public chain without invading the instruction set. We also offer the contract programming language [Minits][minits], a subset of Typescript designed specifically for blockchain smart contract development, which uses LLVM to eventually compile the code into RISC-V binary running in CKB-VM.


### Built-in Interoperation Capability

The UDT instantiated with FCA has cross-chain function natively, and the cross-chain protocol can be used to implement the decentralized cross-chain between the Huobi public chain and its side chain, as well as between the Huobi public chain and the Nervos CKB.

We pass the cross-chain proof in a decentralized relay. The relayer can be a sidechain validator or other third-party users. Direct cross-chain functionality can be achieved between the side chains without relying on the Huobi chain or the Nervos CKB.

### Native Services

Users can deploy smart contracts in two ways. The first is to compile the contract into RISC-V code and dynamically deploy it to the chain. The second is to deploy native contracts written in Rust. The native contract bypasses the virtual machine's interpretation process and directly accesses system resources, which gains more efficient performance.

In the future, the side chain of the Huobi public chain may widely apply the original contract to some business scenarios that require high performance.

[overlord]: https://github.com/cryptape/overlord
[risc-v]: https://www.wikiwand.com/en/RISC-V
[eip-150]: https://docs.google.com/spreadsheets/d/1n6mRqkBz3iWcOlRem_mO09GtSKEKrAsfO7Frgx18pNU/edit#gid=0
[ckb-vm]: https://github.com/nervosnetwork/ckb-vm
[minits]: https://github.com/cryptape/minits
[move]: https://developers.libra.org/docs/move-overview
[ckb-white-paper]: https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0002-ckb/0002-ckb.md