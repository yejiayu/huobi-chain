# RISC-V Service

## 概述

RISC-V service 是一个基于 [`CKB-VM`](https://github.com/nervosnetwork/ckb-vm) 开发的虚拟机服务组件。

该组件内置了一个 [RISC-V](https://riscv.org/) 指令集解释器作为虚拟机。通过该组件，用户可以自由的部署和调用合约，实现强大的自定义功能。
任何支持 [RV64I]((https://riscv.org/specifications/)) 的编译器 (如 [riscv-gcc](https://github.com/riscv/riscv-gcc), [riscv-llvm](https://github.com/lowRISC/riscv-llvm), [Rust](https://github.com/rust-embedded/wg/issues/218)) 生成的可执行文件均可以作为合约使用。

想要了解跟多 CKB-VM 的信息，可以参考 [CKB RFC](https://github.com/nervosnetwork/rfcs/blob/master/rfcs/0003-ckb-vm/0003-ckb-vm.zh.md)。

## RISC-V 运行模型

RISC-V service 使用 64 位的 RISC-V 虚拟机作为 VM 来执行合约。合约直接使用 Linux 的 ELF 可执行文件格式，合约的运行等同于 Linux 环境下一个可执行文件在单核 CPU 下的运行。

```c
#include <pvm.h>

int main() {
    char args[100] = {0};
    uint64_t args_len = 0;
    pvm_load_args(args, &args_len);

    // your contract logics here

    char result[] = "contract_execute_result";
    pvm_ret(result, strlen(result));

    return 0;
}
```

合约运行从合约 ELF 文件中的 main 函数开始执行。当 main 函数返回值为 0 时，认为合约执行成功，否则合约执行失败。

`pvm.h` 中提供了一些合约执行的辅助函数，包括获取函数执行参数、获取交易上下文、操作合约数据等。
- `pvm_load_args` 从交易中获取执行参数，`pvm_ret` 返回执行结果
- 部分辅助函数可以获取交易上下文。例如 `pvm_block_height` 可以获取当前块高度。
- 每个合约有自己独立的状态空间，相当于一个 kv 数据库，可以存储任意的 bytes。用户可以通过 `pvm_get_storage` 和 `pvm_set_storage` 来操作合约的状态。

CKB-VM 仅为单线程模型，合约文件可以自行提供 coroutine 实现，但是在 VM 层不提供 threading。

## 开发语言

理论上任何提供了 RISC-V 后端的语言均可以用来开发合约:

- 可以直接使用标准的 riscv-gcc 以及 riscv-llvm 以 C/C++ 语言来进行开发，编译后的可执行文件直接作为合约来使用。这是目前最成熟的方案，也是我们推荐使用的方案。文档后续的内容和示例均会用这种方法进行合约开发。
- 其他的高级语言 VM 如 duktape 及 mruby 在编译后，也可以用来相应的运行 JavaScript 或者 Ruby 编写的合约。我们在 dev 和 test 环境提供了 duktape 的内置支持，用户可以用 JavaScript 快速编写合约，进行原型开发和 PoC 验证。此方案虚拟机执行开销较大，不建议在生产环境使用。
- 相应的也可以使用 Rust 作为实现语言来编写合约

## 示例

在 service 源代码的 example 和 test 文件夹中有大量的参考示例。本文档也提供了一个[合约开发的教程](./contract_demo)，请读者自行参阅相关章节。

## 创世块配置

```toml
[[services]]
name = 'riscv'
payload = '''
{
    "enable_authorization": true,
    "admins": ["0xcff1002107105460941f797828f468667aa1a2db"],
    "deploy_auth": ["0x9cccacbb8a4b0353d42138613b2db72d6a661cf4"]
}
'''
```

- enable_authorization: 是否开启授权模式，开启后，合约的部署和执行都需要授权
- admins: 服务的管理员地址列表
- deploy_auth: 部署权限预授权地址

## 接口

### 部署合约

```rust
pub enum InterpreterType {
    Binary = 1,
}

pub struct DeployPayload {
    pub code:      String,
    pub intp_type: InterpreterType,
    pub init_args: String,
}

pub struct DeployResp {
    pub address:  Address,
    pub init_ret: String,
}
```

- method: deploy
- 参数
  - code：合约代码，使用 hex 编码
  - intp_type：生产环境目前仅支持 `Binary`，即 ELF 二进制文件格式
  - init_args：初始化参数
- 返回值
  - address：合约地址
  - init_ret：初始化函数调用返回值

### 调用合约

```rust
pub struct ExecPayload {
    pub address: Address,
    pub args:    String,
}
```

- method:
  - 写调用： exec
  - 只读调用：call
- 参数
  - address：调用的合约地址
  - args：合约调用参数
- 返回值：为合约返回的字符串
- 注: call 调用如果有修改状态，会返回错误。

### 获取合约

```rust
pub struct GetContractPayload {
    pub address:      Address,
    pub get_code:     bool,
    pub storage_keys: Vec<String>,
}
```

- method: get_contract
- 参数
  - address: 要获取的合约地址
  - get_code: 是否获取合约的二进制代码
  - storage_keys: 要获取合约内保存的数据，其键值
- 返回值
  - code_hsh: 合约的哈希
  - intp_type: 合约的类型
  - code: 合约二进制代码，可为空
  - storage_values: 对应入参中的合约内保存的数据
  - authorizer: 授权合约可执行的授权人地址，如未授权，则为空

### 授权部署合约/撤销授权/查看授权


```javascript
// payload 示例
{
  addresses: ["0xcff1002107105460941f797828f468667aa1a2db"]
}
```

- method:
  - 授权：grant_deploy_auth
  - 取消授权：revoke_deploy_auth
  - 查看授权：check_deploy_auth
- payload: 均为上述示例
  - addresses: 要授权的地址列表
- 返回值
  - grant_deploy_auth 和 revoke_deploy_auth 无返回值
  - check_deploy_auth 返回待检查列表中有权限的地址

### 授权合约可执行/撤销授权

```rust
pub struct AddressList {
    pub addresses: Vec<Address>,
}
```

- method:
  - 授权： approve_contracts
  - 撤销： revoke_contracts
- payload: 均为合约地址数组
  - addresses: 被授权/撤销的合约地址列表
- 返回值
  - approve_contracts 和 revoke_contracts 成功，无返回值
