# Huobi-chain 入门

<details>
  <summary><strong>Table of Contents</strong></summary>

- [Huobi-chain 入门](#huobi-chain-%e5%85%a5%e9%97%a8)
  - [安装和运行](#%e5%ae%89%e8%a3%85%e5%92%8c%e8%bf%90%e8%a1%8c)
    - [安装依赖](#%e5%ae%89%e8%a3%85%e4%be%9d%e8%b5%96)
      - [** MacOS **](#macos)
      - [** ubuntu **](#ubuntu)
      - [** centos7 **](#centos7)
      - [** archlinux **](#archlinux)
    - [直接下载预编译的二进制文件](#%e7%9b%b4%e6%8e%a5%e4%b8%8b%e8%bd%bd%e9%a2%84%e7%bc%96%e8%af%91%e7%9a%84%e4%ba%8c%e8%bf%9b%e5%88%b6%e6%96%87%e4%bb%b6)
    - [从源码编译](#%e4%bb%8e%e6%ba%90%e7%a0%81%e7%bc%96%e8%af%91)
      - [获取源码](#%e8%8e%b7%e5%8f%96%e6%ba%90%e7%a0%81)
      - [安装 rust](#%e5%ae%89%e8%a3%85-rust)
      - [编译](#%e7%bc%96%e8%af%91)
    - [运行单节点](#%e8%bf%90%e8%a1%8c%e5%8d%95%e8%8a%82%e7%82%b9)
  - [与链进行交互](#%e4%b8%8e%e9%93%be%e8%bf%9b%e8%a1%8c%e4%ba%a4%e4%ba%92)
    - [使用 GraphiQL 与链进行交互](#%e4%bd%bf%e7%94%a8-graphiql-%e4%b8%8e%e9%93%be%e8%bf%9b%e8%a1%8c%e4%ba%a4%e4%ba%92)
    - [使用 muta-cli 与链进行交互](#%e4%bd%bf%e7%94%a8-muta-cli-%e4%b8%8e%e9%93%be%e8%bf%9b%e8%a1%8c%e4%ba%a4%e4%ba%92)
  - [使用示例](#%e4%bd%bf%e7%94%a8%e7%a4%ba%e4%be%8b)

  </details>

## 安装和运行

### 安装依赖

<!-- tabs:start -->

#### ** MacOS **

```
brew install autoconf libtool
```

#### ** ubuntu **

```
apt update
apt install -y git curl openssl cmake pkg-config libssl-dev gcc build-essential clang libclang-dev
```

#### ** centos7 **

```
yum install -y centos-release-scl
yum install -y git make gcc-c++ openssl-devel llvm-toolset-7

# 打开 llvm 支持
scl enable llvm-toolset-7 bash
```

#### ** archlinux **

```
pacman -Sy --noconfirm git gcc pkgconf clang make
```

<!-- tabs:end -->

<!--
### 直接下载预编译的二进制文件

我们会通过 [github releases](https://github.com/HuobiGroup/huobi-chain/releases) 发布一些常用操作系统的预编译二进制文件。如果其中包含你的操作系统，可以直接下载对应的文件。
-->

### 从源码编译

#### 获取源码

通过 git 下载源码：

```
git clone https://github.com/HuobiGroup/huobi-chain.git
```

或者在 [github releases](https://github.com/HuobiGroup/huobi-chain/releases) 下载源码压缩包解压。

#### 安装 rust

参考： <https://www.rust-lang.org/tools/install>

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 编译

```
cd /path/to/huobi-chain
make prod
```

编译完成后的二进制文件在 `target/release/huobi-chain`。

### 运行单节点

```
cd /path/to/huobi-chain

# 使用默认配置运行 huobi-chain
# 如果是直接下载的 binary，请自行替换下面的命令为对应的路径
./target/release/huobi-chain

# 查看帮助
$ ./target/release/huobi-chain  -h
Huobi-chain v0.2.0
Muta Dev <muta@nervos.org>

USAGE:
    huobi-chain [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <FILE>     a required file for the configuration [default: ./config/chain.toml]
    -g, --genesis <FILE>    a required file for the genesis [default: ./config/genesis.toml]
```

## 与链进行交互

链默认在 8000 端口暴露了 GraphQL 接口用于用户与链进行交互。

### 使用 GraphiQL 与链进行交互

打开 <http://127.0.0.1:8000/graphiql> 后效果如下图所示：

![](./static/graphiql.png)

左边输入 GraphQL 语句，点击中间的执行键，即可在右边看到执行结果。

点击右边 Docs 可以查阅接口文档。更多 GraphQL 用法可以参阅 [官方文档](https://graphql.org/)。

### 使用 muta-cli 与链进行交互

我们通过 [muta-sdk](https://github.com/nervosnetwork/muta-sdk-js) 和 nodejs 封装了一个交互式命令行，可以更方便的与 huobi-chain 进行交互。

```bash
$ npm install -g muta-cli

$ muta-cli repl
> await client.getLatestBlockHeight()
2081
> await client.getBlock('0x1')
{ header:
   { chainId:
      'b6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036',
     confirmRoot: [],
     cyclesUsed: [ '0000000000000000' ],
     height: '0000000000000001',
     execHeight: '0000000000000000',
     orderRoot:
      '56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421',
     preHash:
      '87f07b8f60bd6198ba52deacfe9ecf9870198edb60a706a1d0fea1f5df1c6a26',
     proposer: 'f8389d774afdad8755ef8e629e5a154fddc6325a',
     receiptRoot: [],
     stateRoot:
      'f846a8c0af225b0d3a4ea5c90e2adfbf207b0accd9a1046832f84aa92947d1f1',
     timestamp: '000000005e3ebfea',
     validatorVersion: '0000000000000000',
     proof:
      { bitmap: '',
        blockHash:
         '56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421',
        height: '0000000000000000',
        round: '0000000000000000',
        signature: '' },
     validators: [ [Object] ] },
  orderedTxHashes: [] }
```

该 REPL 是基于 nodejs 的封装，你可以使用任何符合 nodejs 语法的语句。

环境中默认注入了一些变量，方便使用：
- `muta_sdk`: 即 muta-sdk 库，更多使用方法可以参考 [muta-sdk 文档](https://nervosnetwork.github.io/muta-sdk-js/)
- `muta`: muta 链的 instance
- `client`: 对链进行 GraphQL 调用的 client
- `wallet`: 根据助记词（默认为随机生成）推导出的钱包
- `accounts`: 根据 wallet 推导出的 20 个账号


## 使用示例

以下使用 muta-cli 对链的常用操作进行简单的示例说明：

```bash
$ muta-cli repl
# 链基础交互
> await client.getLatestBlockHeight()
2081

> client.getBlock('0x1')
{ header:
   { chainId:
      'b6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036',
     confirmRoot: [],
     cyclesUsed: [ '0000000000000000' ],
     height: '0000000000000001',
     execHeight: '0000000000000000',
     orderRoot:
      '56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421',
     preHash:
      '87f07b8f60bd6198ba52deacfe9ecf9870198edb60a706a1d0fea1f5df1c6a26',
     proposer: 'f8389d774afdad8755ef8e629e5a154fddc6325a',
     receiptRoot: [],
     stateRoot:
      'f846a8c0af225b0d3a4ea5c90e2adfbf207b0accd9a1046832f84aa92947d1f1',
     timestamp: '000000005e3eecac',
     validatorVersion: '0000000000000000',
     proof:
      { bitmap: '',
        blockHash:
         '56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421',
        height: '0000000000000000',
        round: '0000000000000000',
        signature: '' },
     validators: [ [Object] ] },
  orderedTxHashes: [] }

# asset service 操作
> const account = accounts[0]

> const service = new muta_sdk.AssetService(client, account)

# 发行资产
> HT = await service.createAsset({name: 'Huobi Token', supply: 1000000000, symbol: 'HT'})
{ name: 'Huobi Token',
  symbol: 'HT',
  supply: 1000000000,
  issuer: '9d1d1bb11c44500603971a245f55a23f65148eee',
  asset_id:
   'e8c2c6606030bc93da018cec5e6400845489b471527d507357b3316ae884a3f3' }

# 发行者即为发交易的账户地址
> account.address
'0x9d1d1bb11c44500603971a245f55a23f65148eee'

# 查询发行者余额
> await client.queryService({serviceName: 'asset', method: 'get_balance', payload: JSON.stringify({asset_id: HT.asset_id, user: account.address})})
{ isError: false,
  ret:
   '{"asset_id":"e8c2c6606030bc93da018cec5e6400845489b471527d507357b3316ae884a3f3","user":"9d1d1bb11c44500603971a245f55a23f65148eee","balance":1000000000}' }

# 转账
> const to = accounts[1].address;

> await service.transfer({asset_id: HT.asset_id, to, value: 100});

# 查看转账结果
> await client.queryService({ serviceName: 'asset', method: 'get_balance', payload: JSON.stringify({asset_id: HT.asset_id, user: account.address})})
{ isError: false,
  ret:
   '{"asset_id":"e8c2c6606030bc93da018cec5e6400845489b471527d507357b3316ae884a3f3","user":"9d1d1bb11c44500603971a245f55a23f65148eee","balance":999999900}' }

> await client.queryService({ serviceName: 'asset', method: 'get_balance', payload: JSON.stringify({asset_id: HT.asset_id, user: to})})
{ isError: false,
  ret:
   '{"asset_id":"e8c2c6606030bc93da018cec5e6400845489b471527d507357b3316ae884a3f3","user":"9b13a4625e63b0c475c4a6f5dabb761d1c315f2b","balance":100}' }

# 链上管理
> admin = muta_sdk.Muta.account.fromPrivateKey('2b672bb959fa7a852d7259b129b65aee9c83b39f427d6f7bded1f58c4c9310c2')

> admin.address
'0xcff1002107105460941f797828f468667aa1a2db'

> metadata_raw = await client.queryService({serviceName: 'metadata', method: 'get_metadata', payload: ''})
{ isError: false,
  ret:
   '{"chain_id":"b6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036","common_ref":"703873635a6b51513451","timeout_gap":20,"cycles_limit":99999999,"cycles_price":1,"interval":999,"verifier_list":[{"address":"f8389d774afdad8755ef8e629e5a154fddc6325a","propose_weight":1,"vote_weight":1}],"propose_ratio":15,"prevote_ratio":10,"precommit_ratio":10}' }

> metadata = JSON.parse(metadata_raw.ret)
{ chain_id:
   'b6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036',
  common_ref: '703873635a6b51513451',
  timeout_gap: 20,
  cycles_limit: 99999999,
  cycles_price: 1,
  interval: 999,
  verifier_list:
   [ { address: 'f8389d774afdad8755ef8e629e5a154fddc6325a',
       propose_weight: 1,
       vote_weight: 1 } ],
  propose_ratio: 15,
  prevote_ratio: 10,
  precommit_ratio: 10 }

# 构造交易修改 interval
> update_metadata_tx = await client.composeTransaction({ method: 'update_interval', payload: { interval: metadata.interval - 1, }, serviceName: 'node_manager' })
{ chainId:
   '0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036',
  cyclesLimit: '0xffff',
  cyclesPrice: '0xffff',
  nonce:
   '0x982547f7d2cbb2045991f0d98c935dff3d7741036717c913b6d56e0bae6ddd4e',
  timeout: '0x0542',
  method: 'update_interval',
  payload: '{"interval":998}',
  serviceName: 'node_manager' }

> hash = await client.sendTransaction(admin.signTransaction(update_metadata_tx))
'4a291505feb75c01769b2d80c57f3db168681b07be1d31bce604fb31f9b0dbe2'

> await client.getReceipt(hash)
{ txHash:
   '4a291505feb75c01769b2d80c57f3db168681b07be1d31bce604fb31f9b0dbe2',
  height: '000000000000036f',
  cyclesUsed: '000000000000f618',
  events:
   [ { data: '{"topic":"Interval Updated","interval":998}',
       service: 'node_manager' } ],
  stateRoot:
   'a052b0485718bc7d1b7f004b57473a4922b0dacb19307ce1c50c2e52a3ed2584',
  response:
   { serviceName: 'node_manager',
     method: 'update_interval',
     ret: '',
     isError: false } }

# 再查一次可以发现 interval 已经改变了
> metadata_raw = await client.queryService({serviceName: 'metadata', method: 'get_metadata', payload: ''})
{ isError: false,
  ret:
   '{"chain_id":"b6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036","common_ref":"703873635a6b51513451","timeout_gap":20,"cycles_limit":99999999,"cycles_price":1,"interval":998,"verifier_list":[{"address":"f8389d774afdad8755ef8e629e5a154fddc6325a","propose_weight":1,"vote_weight":1}],"propose_ratio":15,"prevote_ratio":10,"precommit_ratio":10}' }


# riscv service 演示

# 由于部署合约耗费 cycle 较大，调大 client 的默认 cycleslimit
> client.options.defaultCyclesLimit = '0xffffff'

> const fs = require('fs')

# 在 huobi-chain repo 根目录下，可以读取到下列示例合约
> const code = fs.readFileSync('services/riscv/src/tests/simple_storage')

> const tx = await client.composeTransaction({ method: 'deploy', payload: { intp_type: 'Binary', init_args: '', code: code.toString('hex') }, serviceName: 'riscv' })

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

# 查询 k 这个 key 的值
> await client.queryService({ serviceName: 'riscv', method: 'call', payload: JSON.stringify({ address, args: 'get k' })})
{ isError: false, ret: '""' }

# 发交易，将 k 的值置为 v
> set_tx = await client.composeTransaction({ method: 'exec', payload: { address, args: 'set k v' }, serviceName: 'riscv' })
{ chainId:
   '0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036',
  cyclesLimit: '0xffffff',
  cyclesPrice: '0xffff',
  nonce:
   '0x1811fc291c71184e6a1170cce328135e94072910c30b9f31c861375ae6aaf48a',
  timeout: '0x05a8',
  method: 'exec',
  payload:
   '{"address":"7598a35834c5c1f544edd9ba48013c361f71bf3b","args":"set k v"}',
  serviceName: 'riscv' }
> set_tx_hash = await client.sendTransaction(account.signTransaction(set_tx))
'ec8b475ce6908368e21905ba2f79095bd3cb59dcb110c55be8e2b829fbb3e020'
> await client.getReceipt(set_tx_hash)
{ txHash:
   'ec8b475ce6908368e21905ba2f79095bd3cb59dcb110c55be8e2b829fbb3e020',
  height: '00000000000005a3',
  cyclesUsed: '0000000000000d34',
  events: [],
  stateRoot:
   '6d7beb07e96978344acb2ec9ede1225723a299cbaae68b468ba08295c9604f69',
  response:
   { serviceName: 'riscv',
     method: 'exec',
     ret: '""',
     isError: false } }

# 再次查询，发现 k 的值已经修改
> await client.queryService({ serviceName: 'riscv', method: 'call', payload: JSON.stringify({ address, args: 'get k' })})
{ isError: false, ret: '"v"' }

```