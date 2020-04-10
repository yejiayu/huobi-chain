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
      - [安装 RUST](#%e5%ae%89%e8%a3%85-rust)
      - [编译](#%e7%bc%96%e8%af%91)
    - [运行单节点](#%e8%bf%90%e8%a1%8c%e5%8d%95%e8%8a%82%e7%82%b9)
  - [与链进行交互](#%e4%b8%8e%e9%93%be%e8%bf%9b%e8%a1%8c%e4%ba%a4%e4%ba%92)
    - [使用 GraphiQL 与链进行交互](#%e4%bd%bf%e7%94%a8-graphiql-%e4%b8%8e%e9%93%be%e8%bf%9b%e8%a1%8c%e4%ba%a4%e4%ba%92)
    - [使用 muta-cli 与链进行交互](#%e4%bd%bf%e7%94%a8-muta-cli-%e4%b8%8e%e9%93%be%e8%bf%9b%e8%a1%8c%e4%ba%a4%e4%ba%92)
  - [使用示例](#%e4%bd%bf%e7%94%a8%e7%a4%ba%e4%be%8b)
  - [使用 docker 本地部署多节点链](#%e4%bd%bf%e7%94%a8-docker-%e6%9c%ac%e5%9c%b0%e9%83%a8%e7%bd%b2%e5%a4%9a%e8%8a%82%e7%82%b9%e9%93%be)

  </details>

## 安装和运行

### 安装依赖

<!-- tabs:start -->

#### ** MacOS **

```
$ brew install autoconf libtool
```

#### ** ubuntu **

```
$ apt update
$ apt install -y git curl openssl cmake pkg-config libssl-dev gcc build-essential clang libclang-dev
```

#### ** centos7 **

```
$ yum install -y centos-release-scl
$ yum install -y git make gcc-c++ openssl-devel llvm-toolset-7

# 打开 llvm 支持
$ scl enable llvm-toolset-7 bash
```

#### ** archlinux **

```
$ pacman -Sy --noconfirm git gcc pkgconf clang make
```

<!-- tabs:end -->

### 直接下载预编译的二进制文件

我们会通过 [github releases](https://github.com/HuobiGroup/huobi-chain/releases) 发布一些常用操作系统的预编译二进制文件。如果其中包含你的操作系统，可以直接下载对应的文件。

### 从源码编译

#### 获取源码

通过 git 下载源码：

```
$ git clone https://github.com/HuobiGroup/huobi-chain.git
```

或者在 [github releases](https://github.com/HuobiGroup/huobi-chain/releases) 下载源码压缩包解压。

#### 安装 RUST

参考： <https://www.rust-lang.org/tools/install>

```
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 编译

```
$ cd /path/to/huobi-chain
$ make prod
```

编译完成后的二进制文件在 `target/release/huobi-chain`。

### 运行单节点

```bash
$ cd /path/to/huobi-chain

# 使用默认配置运行 huobi-chain
# 如果是直接下载的 binary，请自行替换下面的命令为对应的路径
./target/release/huobi-chain

# 查看帮助
$ ./target/release/huobi-chain  -h
Huobi-chain v0.3.0
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

我们通过 [muta-sdk](./js_sdk) 和 nodejs 封装了一个交互式命令行，可以更方便的与 huobi-chain 进行交互。

```bash
$ npm install -g muta-cli

$ muta-cli repl
> await client.getLatestBlockHeight()
2081
> await client.getBlock('0x1')
{
  header: {
    chainId: '0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036',
    confirmRoot: [],
    cyclesUsed: [],
    height: '0x0000000000000001',
    execHeight: '0x0000000000000000',
    orderRoot: '0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421',
    preHash: '0xa384912d97e717091537b8fbc126d06387661ef6bc65d16629fb670f1c8a012e',
    proposer: '0xf8389d774afdad8755ef8e629e5a154fddc6325a',
    receiptRoot: [],
    stateRoot: '0x6116242b3f08157d9efc825e8b8a6183592e1bbec8144a445978dec3c60c94f9',
    timestamp: '0x00000170d37db1c6',
    validatorVersion: '0x0000000000000000',
    proof: {
      bitmap: '0x',
      blockHash: '0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421',
      height: '0x0000000000000000',
      round: '0x0000000000000000',
      signature: '0x'
    },
    validators: [ [Object] ]
  },
  orderedTxHashes: [],
  hash: '0x253466c6611ce338b78882777c320990db32ca0da156bec0e7c2e9e1489c5419'
}
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
{
  header: {
    chainId: '0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036',
    confirmRoot: [],
    cyclesUsed: [],
    height: '0x0000000000000001',
    execHeight: '0x0000000000000000',
    orderRoot: '0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421',
    preHash: '0xa384912d97e717091537b8fbc126d06387661ef6bc65d16629fb670f1c8a012e',
    proposer: '0xf8389d774afdad8755ef8e629e5a154fddc6325a',
    receiptRoot: [],
    stateRoot: '0x6116242b3f08157d9efc825e8b8a6183592e1bbec8144a445978dec3c60c94f9',
    timestamp: '0x00000170d37db1c6',
    validatorVersion: '0x0000000000000000',
    proof: {
      bitmap: '0x',
      blockHash: '0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421',
      height: '0x0000000000000000',
      round: '0x0000000000000000',
      signature: '0x'
    },
    validators: [ [Object] ]
  },
  orderedTxHashes: [],
  hash: '0x253466c6611ce338b78882777c320990db32ca0da156bec0e7c2e9e1489c5419'
}

# asset service 操作

# 起链后只有 admin 的账户有 HTTest，由于后续的一些写操作需要以 HTTest 支付手续费，所以这里通过 admin 的私钥创建 account
> const account = muta_sdk.Account.fromPrivateKey('0x2b672bb959fa7a852d7259b129b65aee9c83b39f427d6f7bded1f58c4c9310c2')

> const service = new muta_sdk.AssetService(client, account)

# 发行资产
> MT = await service.create_asset({name: 'Muta Token', supply: 1000000000, symbol: 'MT', precision: 8})
{
  txHash: '0x3b7d58e017906aa09c63a7dadc7a770543750467f9a7cf9dc284a1119504fd03',
  height: '0x0000000000003088',
  cyclesUsed: '0x0000000000005208',
  events: [
    {
      data: '{"id":"0x82273a8a250b5dcc6fd0e6abb28669f46a0118d420fe9da536762d7d458432ac","name":"Muta Token","symbol":"MT","supply":1000000000,"precision":8,"issuer":"0xcff1002107105460941f797828f468667aa1a2db"}',
      service: 'asset'
    }
  ],
  stateRoot: '0x9c0c17a8c035516050ea459a0f5a9e7aee10cb80e9e42d6bd3b46ff16f717eba',
  response: {
    serviceName: 'asset',
    method: 'create_asset',
    ret: {
      id: '0x82273a8a250b5dcc6fd0e6abb28669f46a0118d420fe9da536762d7d458432ac',
      name: 'Muta Token',
      symbol: 'MT',
      supply: 1000000000,
      precision: 8,
      issuer: '0xcff1002107105460941f797828f468667aa1a2db'
    },
    isError: false
  }

# 发行者即为发交易的账户地址
> account.address
'0xcff1002107105460941f797828f468667aa1a2db'

# 查询发行者余额

> await client.queryService({serviceName: 'asset', method: 'get_balance', payload: JSON.stringify({asset_id: MT.responce.net.id, user: account.address})})
{
  isError: false,
  ret: '{"asset_id":"0x82273a8a250b5dcc6fd0e6abb28669f46a0118d420fe9da536762d7d458432ac","user":"0xcff1002107105460941f797828f468667aa1a2db","balance":1000000000}'
}

# 转账
> const to = accounts[1].address;

> await service.transfer({asset_id: MT.responce.net.id, to, value: 100});

# 查看转账结果
> await client.queryService({ serviceName: 'asset', method: 'get_balance', payload: JSON.stringify({asset_id: MT.responce.net.id, user: account.address})})
{
  isError: false,
  ret: '{"asset_id":"0x82273a8a250b5dcc6fd0e6abb28669f46a0118d420fe9da536762d7d458432ac","user":"0xcff1002107105460941f797828f468667aa1a2db","balance":999999900}'
}
> await client.queryService({ serviceName: 'asset', method: 'get_balance', payload: JSON.stringify({asset_id: MT.responce.net.id, user: to})})
{
  isError: false,
  ret: '{"asset_id":"0x82273a8a250b5dcc6fd0e6abb28669f46a0118d420fe9da536762d7d458432ac","user":"0x21409229fb79b996e68a2273b98a305ac0a7a785","balance":100}'
}

# 链上管理
> admin = muta_sdk.Muta.account.fromPrivateKey('0x2b672bb959fa7a852d7259b129b65aee9c83b39f427d6f7bded1f58c4c9310c2')

> admin.address
'0xcff1002107105460941f797828f468667aa1a2db'

> metadata_raw = await client.queryService({serviceName: 'metadata', method: 'get_metadata', payload: ''})
{
  isError: false,
  ret: '{"chain_id":"0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036","common_ref":"0x703873635a6b51513451","timeout_gap":20,"cycles_limit":999999999999,"cycles_price":1,"interval":3000,"verifier_list":[{"bls_pub_key":"0x04188ef9488c19458a963cc57b567adde7db8f8b6bec392d5cb7b67b0abc1ed6cd966edc451f6ac2ef38079460eb965e890d1f576e4039a20467820237cda753f07a8b8febae1ec052190973a1bcf00690ea8fc0168b3fbbccd1c4e402eda5ef22","address":"0xf8389d774afdad8755ef8e629e5a154fddc6325a","propose_weight":1,"vote_weight":1}],"propose_ratio":15,"prevote_ratio":10,"precommit_ratio":10,"brake_ratio":7,"tx_num_limit":20000,"max_tx_size":1048576}'
}

> metadata = JSON.parse(metadata_raw.ret)
{
  chain_id: '0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036',
  common_ref: '0x703873635a6b51513451',
  timeout_gap: 20,
  cycles_limit: 999999999999,
  cycles_price: 1,
  interval: 3000,
  verifier_list: [
    {
      bls_pub_key: '0x04188ef9488c19458a963cc57b567adde7db8f8b6bec392d5cb7b67b0abc1ed6cd966edc451f6ac2ef38079460eb965e890d1f576e4039a20467820237cda753f07a8b8febae1ec052190973a1bcf00690ea8fc0168b3fbbccd1c4e402eda5ef22',
      address: '0xf8389d774afdad8755ef8e629e5a154fddc6325a',
      propose_weight: 1,
      vote_weight: 1
    }
  ],
  propose_ratio: 15,
  prevote_ratio: 10,
  precommit_ratio: 10,
  brake_ratio: 7,
  tx_num_limit: 20000,
  max_tx_size: 1048576
}

# 构造交易修改 interval
> update_metadata_tx = await client.composeTransaction({ method: 'update_interval', payload: { interval: metadata.interval - 1, }, serviceName: 'node_manager' })
{
  chainId: '0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036',
  cyclesLimit: '0xffff',
  cyclesPrice: '0xffff',
  nonce: '0x891c39980034dc5d5dc0cd38c5fd48a78a6499fb2976fa230bd8df300df39334',
  timeout: '0x35b4',
  method: 'update_interval',
  payload: '{"interval":2999}',
  serviceName: 'node_manager'
}

> hash = await client.sendTransaction(admin.signTransaction(update_metadata_tx))
'0x9e360cc732328cabe844290cd24d3eb74060125c7c10f67736760aec67ddc4dd'

> await client.getReceipt(hash)
{
  txHash: '0x9e360cc732328cabe844290cd24d3eb74060125c7c10f67736760aec67ddc4dd',
  height: '0x00000000000035a6',
  cyclesUsed: '0x000000000000f618',
  events: [
    {
      data: '{"topic":"Interval Updated","interval":2999}',
      service: 'node_manager'
    }
  ],
  stateRoot: '0xfaf9040beada4e7390a3f23e0e210ad050c04b74a20195dd74bbb5f01dedb14b',
  response: {
    serviceName: 'node_manager',
    method: 'update_interval',
    ret: '',
    isError: false
  }
}

# 再查一次可以发现 interval 已经改变了
> metadata_raw = await client.queryService({serviceName: 'metadata', method: 'get_metadata', payload: ''})
{
  isError: false,
  ret: '{"chain_id":"0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036","common_ref":"0x703873635a6b51513451","timeout_gap":20,"cycles_limit":999999999999,"cycles_price":1,"interval":2999,"verifier_list":[{"bls_pub_key":"0x04188ef9488c19458a963cc57b567adde7db8f8b6bec392d5cb7b67b0abc1ed6cd966edc451f6ac2ef38079460eb965e890d1f576e4039a20467820237cda753f07a8b8febae1ec052190973a1bcf00690ea8fc0168b3fbbccd1c4e402eda5ef22","address":"0xf8389d774afdad8755ef8e629e5a154fddc6325a","propose_weight":1,"vote_weight":1}],"propose_ratio":15,"prevote_ratio":10,"precommit_ratio":10,"brake_ratio":7,"tx_num_limit":20000,"max_tx_size":1048576}'
}

# riscv service 演示

# 由于部署合约耗费 cycle 较大，调大 client 的默认 cycleslimit
> client.options.defaultCyclesLimit = '0xffffff'

> const fs = require('fs')

# 在 huobi-chain repo 根目录下，可以读取到下列示例合约
> const code = fs.readFileSync('services/riscv/src/tests/simple_storage')

> const tx = await client.composeTransaction({ method: 'deploy', payload: { intp_type: 'Binary', init_args: '', code: code.toString('hex') }, serviceName: 'riscv' })

> txHash = await client.sendTransaction(account.signTransaction(tx))
'0xa9bc79abe6bc087d0530561c9d1893c60bf6c622ce087e08f727a1a17502e757'

> receipt = await client.getReceipt(txHash)
{
  txHash: '0xa9bc79abe6bc087d0530561c9d1893c60bf6c622ce087e08f727a1a17502e757',
  height: '0x0000000000003bdf',
  cyclesUsed: '0x0000000000019550',
  events: [],
  stateRoot: '0xdc8f9d29d8b53177b9b3255a9eaf1e5fcafe6d90a58f5e031a67ca92c8a47f5f',
  response: {
    serviceName: 'riscv',
    method: 'deploy',
    ret: '{"address":"0xe73da76a97bb8d052b691f903382ef11efe11e51","init_ret":""}',
    isError: false
  }
}

> address = JSON.parse(receipt.response.ret).address
'0xe73da76a97bb8d052b691f903382ef11efe11e51'

# 查询 k 这个 key 的值
> await client.queryService({ serviceName: 'riscv', method: 'call', payload: JSON.stringify({ address, args: 'get k' })})
{ isError: false, ret: '""' }

# 发交易，将 k 的值置为 v
> set_tx = await client.composeTransaction({ method: 'exec', payload: { address, args: 'set k v' }, serviceName: 'riscv' })
{
  chainId: '0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036',
  cyclesLimit: '0xffffff',
  cyclesPrice: '0xffff',
  nonce: '0xa2bc6e40c6988ed7e9404a03fed8796ceb9c8080286b1d7edb4be14bb29feca1',
  timeout: '0x3c10',
  method: 'exec',
  payload: '{"address":"0xe73da76a97bb8d052b691f903382ef11efe11e51","args":"set k v"}',
  serviceName: 'riscv'
}
> set_tx_hash = await client.sendTransaction(account.signTransaction(set_tx))
'0x64f5d4d55ea3b2db0103c9eee9ccf1a0d0eaf7e1b4f669de037b1b7102554441'
> await client.getReceipt(set_tx_hash)
{
  txHash: '0x64f5d4d55ea3b2db0103c9eee9ccf1a0d0eaf7e1b4f669de037b1b7102554441',
  height: '0x0000000000003c07',
  cyclesUsed: '0x0000000000000d28',
  events: [],
  stateRoot: '0x4d29fdc6e5b707269b64c56ab1d0a8c359117a8ea8c2b2b58a431e39648e2690',
  response: { serviceName: 'riscv', method: 'exec', ret: '""', isError: false }
}

# 再次查询，发现 k 的值已经修改
> await client.queryService({ serviceName: 'riscv', method: 'call', payload: JSON.stringify({ address, args: 'get k' })})
{ isError: false, ret: '"v"' }
```

## 使用 docker 本地部署多节点链

需要预先安装 [docker](https://www.docker.com/)。

1. 构建 docker 镜像

```bash
$ cd /path/to/huobi-chain

$ make docker-build
```

2. 运行 docker compose 命令起链

```bash
$ docker compose -f devtools/docker-compose/bft-4-node.yaml up
```

Docker compose 启动 4 个共识节点，分别暴露 GraphQL 本地端口 8001、8002、8003、8004，节点的详细配置信息可前往 `devtools/docker-compose` 目录查看。