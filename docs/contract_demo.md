# RISC-V C 合约开发教程

> 目前测试链对于部署合约做了白名单控制，用户如想体验，可以在本地部署一条私有测试链进行测试

## 概述

RISC-V service 为 Huobi Chain 提供了一个支持 RISC-V 指令集的虚拟机服务。
用户可以通过该服务自行部署和运行合约，实现强大的自定义功能。

一个 RISC-V 合约的本质是一个 Linux 的 ELF 可执行文件，使用虚拟机运行该合约等同于 Linux 环境下在单核 CPU 下运行这个可执行文件。

理论上任何提供了 RISC-V 后端的语言均可以用来开发合约。就生成代码的体积和质量（运行过程中 cycle 的消耗）而言，目前最成熟的工具是 riscv-gcc。

本文将以 C 语言开发的一个 ERC20 Token 和 Bank 合约为例，为你展示如何编写、部署、调用、测试一个 RISC-V 合约。

## Echo 合约示例

我们首先来看一个简单的 echo 合约：

```c
#include <pvm.h>

int main() {
    char args[100] = {0};
    uint64_t args_len = 0;
    pvm_load_args(args, &args_len);

    pvm_ret(args, args_len);
	return 0;
}
```

该合约的作用是将参数的内容原样返回。

将该 C 代码通过 riscv-gcc 编译生成的二进制文件即为我们的合约。

运行合约时，从 main 函数开始。当 main 函数返回值为 0 时，认为合约执行成功，否则合约执行失败。

注意，这个合约中我们引入了 `pvm.h`，使用了其中的 `pvm_load_args` 和 `pvm_ret` 函数。
`pvm.h` 这个文件中包含了我们与链交互所需要的所有函数。这些函数是通过系统调用实现的，我们将在下节进行详细讲解。

## 系统调用

由于 CKB-VM 只是一个 RISC-V 指令集解释器。要实现合约的复杂逻辑，必然要与链进行交互，如解析参数，返回结果，获取链/交易上下文，操作合约状态等。因此我们在 risc-v service 中用系统调用实现了这些交互功能。

下面是 `pvm_load_args` 函数的实现。调用该函数时，虚拟机会根据寄存器的状态，调用虚拟机外部对应功能的实现函数，并将实现结果通过寄存器和内存写回虚拟机。例如下面的 `pvm_load_args` 调用时，会将交易中的合约调用参数写到虚拟机内部的内存，然后将内存起始地址和参数长度写到对应的寄存器。

```c
static inline long
__internal_syscall(long n, long _a0, long _a1, long _a2, long _a3, long _a4, long _a5)
{
    register long a0 asm("a0") = _a0;
    register long a1 asm("a1") = _a1;
    register long a2 asm("a2") = _a2;
    register long a3 asm("a3") = _a3;
    register long a4 asm("a4") = _a4;
    register long a5 asm("a5") = _a5;
    register long syscall_id asm("a7") = n;
    asm volatile ("scall": "+r"(a0) : "r"(a1), "r"(a2), "r"(a3), "r"(a4), "r"(a5), "r"(syscall_id));
    return a0;
}

#define syscall(n, a, b, c, d, e, f) \
    __internal_syscall(n, (long)(a), (long)(b), (long)(c), (long)(d), (long)(e), (long)(f))

int pvm_load_args(uint8_t *data, uint64_t *len)
{
    return syscall(SYSCODE_LOAD_ARGS, data, len, 0, 0, 0, 0);
}
```

RISC-V service 中提供的系统调用有 4 类：
- debug 工具。`pvm_debug` 提供了虚拟机内部的 debug 工具，用户可以打印一段任意的 bytes，方便合约进行调试
- 入参出参。`pvm_load_args` 和 `pvm_ret` 分别提供了合约的入参和出参功能，用户通过前者获取合约调用参数，通过后者返回合约执行结果。合约的入参和出参均为任意的 bytes
- 获取交易上下文。例如通过 `pvm_caller` 获取调用该函数的地址，`pvm_block_height` 获取当前块高度。
- 链操作：
  - `pvm_set_storage` 和 `pvm_get_storage` 可以用来操作合约的状态空间。每个合约拥有独立的地址，在该地址下拥有独立的状态空间，可以把这个状态空间理解成一个 kv 数据库，用户可以在里面保存任意的数据。合约只能访问和修改自己状态空间内的数据。可以把这个状态空间类比理解成以太坊的 contract storage。
  - `pvm_contract_call` 可以用来调用其它 RISC-V 合约，`pvm_service_call` 可以用来调用 huobi-chain 的其它 build-in service。

所有的系统调用函数都在 [`pvm.h` 文件](https://github.com/HuobiGroup/huobi-chain/blob/master/services/riscv/src/vm/c/pvm.h)中，里面有详细的函数文档，读者可以自行查阅。

## ERC20 和 Bank 合约代码

理解了系统调用后，我们来看一个真实的 ERC20 合约和 Bank 合约的例子。

我们将源码放到了 GitHub 上，读者可以将示例代码下载到本地进行查看和交互。

```
git clone https://github.com/nervosnetwork/riscv-contract-tutorials.git
cd riscv-contract-tutorials/bank
```

[ERC20 合约](https://github.com/nervosnetwork/riscv-contract-tutorials/blob/master/bank/erc20.c) 是一个符合 ERC20 标准的 token 合约。本合约仅作为说明合约功能之用，读者如有发行自定义资产的需求，建议使用 huobi-chain 的原生资产模块，asset service。

[Bank 合约](https://github.com/nervosnetwork/riscv-contract-tutorials/blob/master/bank/bank.c)实现了存钱、取钱、查余额等功能，展示了合约如何与其它合约相互操作（例如存钱时需要调用 ERC20 合约的 transfer_from 功能），该合约本身并没有什么现实意义，但可以很容易扩展成一个类似 EtherDelta 这样的 DEX 合约。

合约说明：
- 序列化：由于合约的入参出参均为任意的 bytes，对于功能复杂的合约，我们可能需要引入一些序列化方法。在上述的代码中，我们使用的是 JSON 格式，因为 JSON 使用广泛，无 schema 限制，且可读性较好。用户也可以根据自己的需求（速度、可读性、大小），选择适合的序列化方案，如 rlp，protobuf，thrift，msgpack 等。
- 函数分发：合约执行的统一入口为 main 函数，用户如想在一个合约中实现许多不同的功能，可以自行在 main 函数中进行函数分发。上述示例中，即是通过 method 字段的内容来路由到不同的函数进行处理。
- 除了系统合约外，我们还用到了很多其他的 C 语言库来帮助开发，具体内容可以参见 deps 文件夹。

## 编译

我们使用 riscv-gcc 来将 C 源码编译成二进制文件。由于 riscv-gcc 工具编译较为复杂，我们提供了打包好的 docker 镜像供读者使用。编译示例如下：

读者可以在 bank 文件夹中运行：

```
make bin_docker
```

命令来使用 docker 进行编译，在 bin 文件夹下得到的两个二进制文件即为我们的合约。

具体的编译可以参考项目的 Makefile:

```Makefile
TARGET := riscv64-unknown-elf
CC := $(TARGET)-gcc
LD := $(TARGET)-gcc
CFLAGS := -Os -DCKB_NO_MMU -D__riscv_soft_float -D__riscv_float_abi_soft
LDFLAGS := -lm -Wl,-static -fdata-sections -ffunction-sections -Wl,--gc-sections -Wl,-s
CURRENT_DIR := $(shell pwd)
DOCKER_BUILD := docker run --rm -it -v $(CURRENT_DIR):/src nervos/ckb-riscv-gnu-toolchain:xenial bash -c
DEPS := $(CURRENT_DIR)/deps
MID := $(CURRENT_DIR)/middle
BIN := $(CURRENT_DIR)/bin

libpvm.a:
	$(CC) -I$(DEPS) -c $(DEPS)/pvm.c -o $(MID)/pvm.o
	$(CC) -I$(DEPS) -c $(DEPS)/UsefulBuf.c -o $(MID)/UsefulBuf.o
	$(CC) -I$(DEPS) -c $(DEPS)/pvm_structs.c -o $(MID)/pvm_structs.o
	ar rcs $(MID)/libpvm.a $(MID)/UsefulBuf.o $(MID)/pvm_structs.o $(MID)/pvm.o

bin: libpvm.a
	$(CC) -I$(DEPS) $(DEPS)/cJSON.h $(DEPS)/cJSON.c erc20.c $(MID)/libpvm.a $(LDFLAGS) -o $(BIN)/erc20.bin
	$(CC) -I$(DEPS) $(DEPS)/cJSON.h $(DEPS)/cJSON.c bank.c $(MID)/libpvm.a $(LDFLAGS) -o $(BIN)/bank.bin

bin_docker:
	$(DOCKER_BUILD) "cd /src && make bin"

clear:
	@rm -rf middle/*
```

## 部署

RISC-V service 的部署合约接口签名如下：

```rust
pub enum InterpreterType {
    Binary = 1,
    #[cfg(debug_assertions)]
    Duktape = 2,
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

其中：

- 参数
  - code：合约代码，使用 hex 编码
  - intp_type：生产环境目前仅支持 `Binary`，即 ELF 二进制文件格式，dev 和 test 环境可以使用 `Duktape`，即使用 js 代码作为合约代码
  - init_args：初始化参数
- 返回值
  - address：合约地址
  - init_ret：初始化函数调用返回值

我们使用 muta-client 来进行操作：

```bash
# 部署前请先在本地起一条单节点的链

$ muta-cli repl
# 由于部署合约耗费 cycle 较大，调大 client 的默认 cycleslimit
> client.options.defaultCyclesLimit = '0xffffff'

> const fs = require('fs')
> const erc20 = fs.readFileSync('bin/erc20.bin')

> const tx = await client.composeTransaction({ method: 'deploy', payload: { intp_type: 'Binary', init_args: '{"method":"init","name":"bitcoin","symbol":"BTC","supply":1000000000}', code: erc20.toString('hex') }, serviceName: 'riscv' })

> const account = accounts[0]
> txHash = await client.sendTransaction(account.signTransaction(tx))
'4a8baf53d59bed2ef016526030203a0d15ed838e0dfcc717495b56142cc0c77a'

> receipt = await client.getReceipt(txHash)
{ txHash:
   '4a8baf53d59bed2ef016526030203a0d15ed838e0dfcc717495b56142cc0c77a',
  height: '0000000000000510',
  cyclesUsed: '0000000000024e50',
  events: [],
  stateRoot:
   '64b18096bf322b74d3f6ae206d5fe3154fa19a876fe849130f9b5f70d627c850',
  response:
   { serviceName: 'riscv',
     method: 'deploy',
     ret:
      '{"address":"7598a35834c5c1f544edd9ba48013c361f71bf3b","init_ret":""}',
     isError: false } }

> address = JSON.parse(receipt.response.ret).address
'7598a35834c5c1f544edd9ba48013c361f71bf3b'
```

## 交互

RISC-V service 提供了两种 `exec` 和 `call` 两个交互接口。前者为写接口，可以操作链上数据，需要通过发交易，打包执行，后者为查询接口，可以通过链的 query 功能直接调用。

示例（继续使用刚才的 client）：

```
# 查询接口
> await client.queryService({ serviceName: 'riscv', method: 'call', payload: JSON.stringify({ address, args: JSON.stringify({method: 'total_supply'}) })})
{ isError: false, ret: '"1000000000"' }

# 发交易
> payload = JSON.stringify({ address, args: JSON.stringify({method: 'transfer', recipient: '0000000000000000000000000000000000000000', amount: 100})})
> tx = await client.composeTransaction({ method: 'exec', payload, serviceName: 'riscv' })
> txHash = await client.sendTransaction(account.signTransaction(tx))
> receipt = await client.getReceipt(txHash)
{ txHash:
   '9f18a395972012817c68611e86e182af72f33964ec86c629c8727a9ec1a79daa',
  height: '0000000000000386',
  cyclesUsed: '0000000000072306',
  events: [],
  stateRoot:
   '7f95c8a6d338fbd64de764cdf1870cc60696c4e69af1656de279d7cded6026ed',
  response:
   { serviceName: 'riscv',
     method: 'exec',
     ret: '""',
     isError: false } }
```

## 测试

测试是合约开发的重要环节。目前我们可以通过两种方式进行合约测试。

如果读者熟悉 RUST，可以通过写 RUST test 或者 binary，来模拟交易直接执行合约代码。
使用这种方法进行测试无需起链。

参见：<https://github.com/HuobiGroup/huobi-chain/blob/master/services/riscv/src/tests/mod.rs#L45>

另一种方法是，在本地起一条单节点测试链，通过 muta-sdk 进行部署和调用，在其中加入测试逻辑，保证代码的正确性。

参见：<https://github.com/nervosnetwork/riscv-contract-tutorials/blob/master/bank/riscv_demo.js>

基于本地测试链运行该脚本可以方便的对合约进行测试。

未来我们也将提供一些独立的组件，来帮助用户更方便的开发、测试合约。如果读者有兴趣，欢迎加入社区进行贡献。

## FAQ
