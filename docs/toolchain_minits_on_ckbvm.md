# 在 CKB-VM 上运行 TypeScript

- [在 CKB-VM 上运行 TypeScript](#在-ckb-vm-上运行-typescript)
  - [太长不看](#太长不看)
    - [构建 ckb-vm](#构建-ckb-vm)
    - [构建 minits](#构建-minits)
    - [构建 LLVM](#构建-llvm)
    - [构建 riscv-gnu-toolchain](#构建-riscv-gnu-toolchain)
    - [测试代码](#测试代码)
  - [LLVM RISC-V 后端支持情况](#llvm-risc-v-后端支持情况)
  - [坑](#坑)
    - [**Bug 24389** **- can't link soft-float modules with double-float modules**](#bug-24389---cant-link-soft-float-modules-with-double-float-modules)
    - [构建失败](#构建失败)
  - [一些有用的链接](#一些有用的链接)

[CKB-VM](https://github.com/nervosnetwork/ckb-vm) 是一个能够运行 RISC-V 指令集的虚拟机。要是它支持 TypeScript ，我们可以将 TypeScript 编译到 RISC-V 指令集。这里我们可以借助 [minits](https://github.com/cryptape/minits) 与 [LLVM](https://llvm.org/) 来实现，也就是`TypeScript -- minits --> LLVM IR -- LLVM-RISCV --> RISC-V` 这样一条编译路径最终获得 RISC-V 指令集。

## 太长不看

### 构建 ckb-vm

```shell
 git clone https://github.com/nervosnetwork/ckb-vm
 cd ckb-vm
 cargo build
```

### 构建 minits

```shell
git clone https://github.com/cryptape/minits
cd minits
npm install 
npm config set cmake_LLVM_DIR $(path-to-llvm/bin/llvm-config --cmakedir)
npm run build
```

### 构建 LLVM

```shell
git clone https://github.com/llvm/llvm-project.git
cd llvm-project
mkdir build
cd build
cmake -DLLVM_TARGETS_TO_BUILD="X86;RISCV;PowerPC" -DLLVM_ENABLE_PROJECTS=clang -G "Unix Makefiles" ../llvm
cmake --build .
```

### 构建 riscv-gnu-toolchain

```shell
git clone --recursive https://github.com/riscv/riscv-gnu-toolchain
sudo apt-get install autoconf automake autotools-dev curl libmpc-dev libmpfr-dev libgmp-dev gawk build-essential bison flex texinfo gperf libtool patchutils bc zlib1g-dev libexpat-dev
./configure --prefix=/opt/riscv
make
```

### 测试代码

```typescript
// fib.ts
function fibo(n: number): number {
    if (n < 2) {
        return n;
    }
    return fibo(n - 1) + fibo(n - 2);
}

function main(argc: number, argv: string[]): number {
    return fibo(13);
}
```

```rust
// demo.rs
use bytes::Bytes;
use std::io::Read;

fn main() {
    let args: Vec<Bytes> = std::env::args().map(|a| a.into()).collect();

    let mut file = std::fs::File::open("examples/main").unwrap();
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).unwrap();
    let buffer = Bytes::from(buffer);

    let r = ckb_vm::run::<u64, ckb_vm::SparseMemory<u64>>(&buffer, &args[..]).unwrap();
    println!("result is {:?}", r);
}
```

```shell
# minits
node ./build/main/index.js build main.ts -o main.ll
clang -O -c --target=riscv64 main.ll
rriscv64-unknown-elf-gcc -o main main.o

# copy main to ckb-vm/examples
cargo run --example demo 
```

## LLVM RISC-V 后端支持情况

目前 *LLVM 8.x*  RISC-V 后端还处于实验状态，但已经出现在 [Target列表](https://llvm.org/svn/llvm-project/llvm/trunk/lib/Target/RISCV/) 中。[在 *LLVM 9.0* 中 *RISC-V* 支持将有望从实验性功能变更为官方功能](https://lists.llvm.org/pipermail/llvm-dev/2019-July/133724.html)，并已经加入 [RC 版本的 Release Note](http://prereleases.llvm.org/9.0.0/rc6/docs/ReleaseNotes.html#changes-to-the-riscv-target) 。[要开启实验性的功能只能从源码构建](https://stackoverflow.com/questions/46905464/how-to-enable-a-llvm-backend)，目前需要构建需要加上 `-DLLVM_TARGETS_TO_BUILD="RISCV"`参数才能构建出有RISC-V的backend。



## 坑

### [**Bug 24389**](https://sourceware.org/bugzilla/show_bug.cgi?id=24389) **- can't link soft-float modules with double-float modules**

在 Mac 上直接使用 brew 安装的版本可能还未修复这个bug，只能在 **Ubuntu 18** 下从源码构建

### 构建失败

```
cmake -DLLVM_TARGETS_TO_BUILD="RISCV" -DLLVM_ENABLE_PROJECTS=clang -G "Unix
 Makefiles" ../llvm

collect2: fatal error: ld terminated with signal 9 [Killed]
compilation terminated.
tools/lto/CMakeFiles/LTO.dir/build.make:167: recipe for target 'lib/libLTO.so.10svn' failed
make[2]: *** [lib/libLTO.so.10svn] Error 1
make[2]: *** Deleting file 'lib/libLTO.so.10svn'
CMakeFiles/Makefile2:10139: recipe for target 'tools/lto/CMakeFiles/LTO.dir/all' failed
make[1]: *** [tools/lto/CMakeFiles/LTO.dir/all] Error 2
Makefile:151: recipe for target 'all' failed
make: *** [all] Error 2
```

## 一些有用的链接

- [LLVM Target Triple](https://llvm.org/doxygen/classllvm_1_1Triple.html)
- [使用Clang 交叉编译](https://clang.llvm.org/docs/CrossCompilation.html)