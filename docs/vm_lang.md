# 合约编程

由于合约虚拟机使用了 RISCV 指令集, 因此任何可以编译到 RISCV 的编程语言都可以作为合约的编程语言. 虚拟机使用的是 rv64imc 架构, 它基于 RV64I ISA 核心, 具有 M 标准扩展用于整数乘法和除法, 以及 C 标准扩展用于 RCV(RISC-V压缩指令). 注意的是, 虚拟机不支持浮点指令.

合约开发者可以根据自己的爱好来选择合约开发语言, 或者使用我们专门为 RISCV 虚拟机编写的编程语言: minits. minits 是一个以 LLVM 为后端的 TypeScript 静态编译器, 它可以将 TypeScript 代码编译为 RISCV 指令, 它拥有 TypeScript 的对开发者友好的语法, 同时拥有不亚于 C 语言的执行性能. 可访问 [https://github.com/cryptape/minits](https://github.com/cryptape/minits) 来获取关于 minits 更多的信息.

# Minits 最小运行模型

任何合约都已一个 main 函数作为其入口函数, 同时返回一个退出码. 如果退出码非 0, 则意味着合约调用失败.

```ts
function main(argc: number, argv: string[]): number {
    return 0
}
```

# Example

下面的代码是一个 SimpleStorage 合约的例子. 该合约允许使用者存储或读取一对 K/V 值. 我们未来会将 syscall, set_storage, get_storage 这些函数以一个 SDK 形式对外提供, 但目前直接写在合约代码中更有利于开发者理解. syscall 是一个特殊的函数, 它允许合约与链上数据进行交互, 比如查询当前链高度, 获取某个账号的余额等.

```ts
// A simplestorage contract for blockchain.

const STORAGE_SET = 2180;
const STORAGE_GET = 2181;
const RET = 2182;

function syscall(n: number, a: any, b: any, c: any, d: any, e: any, f: any): number {
    return 0;
}

function set_storage(k: string, v: string): number {
    return syscall(STORAGE_SET, k, v, 0, 0, 0, 0);
}

function get_storage(k: string): string {
    let v = "";
    syscall(STORAGE_GET, k, v, 0, 0, 0, 0);
    return v
}

function ret(d: string): number {
    return syscall(RET, d, 0, 0, 0, 0, 0);
}

function main(argc: number, argv: string[]): number {
    if (argc == 1) {
        return 1;
    }
    switch (argv[1]) {
        case "get":
            const v = get_storage(argv[2]);
            ret(v);
            return 0;
        case "set":
            set_storage(argv[2], argv[3]);
            return 0;
        default:
            return 1;
    }
}
```