# 火币公链智能合约开发文档

## RISC-V
risc-v是一种指令集架构(instruction set architecture,ISA),可以把他类比成x86指令集.

[risc-v isa spec](https://content.riscv.org/wp-content/uploads/2016/06/riscv-spec-v2.1.pdf)

risc-v指令集可以被运行在多种硬件软件之上.能够支持risc-v指令集软件又叫risc-v vm.

## RISC-V Compiler & Toolchain

risc-v编译器以及工具链由risc-v官方提供:
[Github链接](https://github.com/riscv/riscv-gnu-toolchain)

Ckb将其制作成docker image,方便使用,使之免于手动安装相关环境:
[Docker Hub链接](https://hub.docker.com/r/nervos/ckb-riscv-gnu-toolchain/tags)

请使用nervos/ckb-riscv-gnu-toolchain:xenial或nervos/ckb-riscv-gnu-toolchain:bionic

其中包含的toolchain版本是:8.3.0
```shell script
# riscv64-unknown-elf-gcc --version
riscv64-unknown-elf-gcc (GCC) 8.3.0
```

## RISC-V vm

火币公链采用ckb-vm是一个risc-v vm的实现,内置于火币公链内.
pvm是一个在ckb-vm环境内,为智能合约提供访问ckb-vm外的环境的函数库,主要用于与链做交互.

## 如何使用

1. 用C语言编写合约.
    - 引入pvm.h,通过内置的方法来访问有关于的链的数据,包括:
        - block信息
        - tx信息
        - 当前环境调用上下文
        - 读取/存储storage
        - 调用其他合约
        - 调用其他service
        - 打印调试信息和断言
    - 引入pvm_extend.h,获得handy方法
    
2. 调用toolchain编译你的合约.
    - 准备好toolchain所需的参数,配置好include目录,合约代码目录,并挂载到docker container内.
    - 可以使用pvm.c重新编译pvm然后静态链接,也可以使用编译好的libpvm.a进行静态链接.
        - 不要使用动态链接,否则其他节点没有你的动态链接库将导致合约运行失败.
    - 保存好编译完成的合约二进制文件.
    
3. 将二进制文件读入,向riscv service发起deploy请求,DeployPayload中的code是二进制文件的hex编码.最终获得合约地址.

4. 使用上一步生成的地址,通过API调用你的合约.

## 其他
参考contract/Makefile.

1. cd到contract目录

2. make build-all

3. 查看target目录