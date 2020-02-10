# JS-SDK

JS-SDK 是官方推出的基于 Javascript 的 SDK 工具。它包装了节点信息，可以用 Graphql RPC 方法进行管理账户以及与内建Service 交互。
JS-SDK 之于 Huobi Chain 类似 web3.js 之于 Ethereum。

## Install

```
$ npm install muta-sdk
```

## Modules

详情请查看[SDK 文档](https://nervosnetwork.github.io/muta-sdk-js/)。

1. Client：屏蔽了 GraphQL 的细节，帮助开发者更方便地和链做 GraphQL API 交互。
2. Account：进行 Huobi Chain 的账户管理，一个账户包含了这个账户的私钥,公钥以及地址。
3. Wallet：Huobi Chain 的钱包功能，可以管理多个Account。
4. Builtin-Service：提供对于 Huobi Chain 内置 Service 的直接通行。类似与对以太坊智能合约进行合约级别的 API 通信。

## Examples

在这里例子中，将带你完整的体验上面的每一个模块，从如何构建一个Muta对象，用以和链开始交互，到创建一个账户，再到构建 Client 对象，和 Service 进行交互。

- Step 1：构建一个 Muta 对象，用以和链开始交互
- Step 2：创建分层确定性 HD 钱包，来管理你的账户
- Step 3：创建一个 Account，来管理账户的公私钥对
- Step 4：构建 Client 对象，正式和链上的 Service 进行数据交互
- Step 5：通过使用 AssetService API，直接和 AssetService 交互

#### Step 1：构建一个 muta 对象，用以和链进行交互

```js
const muta = new Muta({
    /**
     * 通常是在genesis.toml里包含有默认的chain_id
     * 在这个例子中我们假设0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036是你要访问的链的ChainId
     */
    chainId:
      '0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036',

    /**
     *  接下来我们给出GraphQL API uri. endpoint是用来和链进行rpc交互的uri,
     *  http://127.0.0.1:8000/graphql是默认的endpoint是用来和链进行rpc交互的uri,
     *  你可以在config.toml文件下的[graphql]部分找到endpoint的配置
     */
    endpoint: 'http://127.0.0.1:8000/graphql',

    /**
     * timeout_gap表示一笔交易发出后，最多允许几个块的延迟.如果随着链的进行,block超出了
     * timeout_gap的设置但是交易仍然没有上链,那么这笔交易就被认为无效了.
     * 比起以太坊的txpool的不确定性,Huobi-chain提供了tx及时性的检测和保障.
     * timeoutGap并没有默认值,但是js-sdk预设为20,你可以所以更改
     */
    timeoutGap: DEFAULT_TIMEOUT_GAP,
  });
```

当然,如果你通过[快速入门](./getting_started.md)起了一条默认配置的链，并且现在你只是想跑通本文档的例子，你可以直接执行下面的指令。

```
  /**
   * 因为测试链的参数基本一致,所以上面的参数一般不会修改,那么下面的语句和上面的逻辑是一样的
   */
  const mutaByDefaultConfig = Muta.createDefaultMutaInstance();
```

好的，现在你已经了解了 Muta 类了，非常简单，接下来让我们看看分层确定性钱包吧。

#### Step 2：创建分层确定性HD钱包，来管理你的账户

首先,如果你不了解 HD 钱包(分层确定性钱包)，请先了解：
1. [bip32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)
2. [bip39](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
3. [bip44](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki)

我们需要先获得 HDWallet 的类型，注意，它只是构造函数，不是对象：

```js
const Wallet = Muta.hdWallet;
```

使用 HD 钱包，通常你需要一组 12 个字的助记词，你可以用已有的助记词，或者通过下面的方法生成助记词，
当然，该组助记词会用来生成 seed 种子，然后构建 HDWallet 的 masterNode。

```js
const mnemonicWords = Wallet.generateMnemonic();
```

然后你可以使用助记词来构建一个 HDWallet 了，这里使用的是我们刚才随机生成的助记词，当然你也可以用已有的：

```js
const hdWallet = new Wallet(mnemonicWords);
const hdWallet = new HDWallet(
    'drastic behave exhaust enough tube judge real logic escape critic horror gold'
  );
```

仅接着，可以通过创建的 HDWallet 来派生子秘钥了。
根据 bip44 的规范，我们的派生路径被设定为：
`m/44'/${COIN_TYPE}'/${accountIndex}'/0/0`
其中 `COIN_TYPE = 918`，accountIndex 就是需要派生的账户的索引。

```js
const account = hdWallet.deriveAccount(2);//我们派生accountIndex=2的账户
```

下面创建一个 Account，用来来管理账户的公私钥对。

#### Step3：创建 Account，来管理账户的公私钥对。

Account 包含了一对公私钥对，以及他派生出来的地址。Huobi Chain 使用 secp256k1 作为签名曲线.

通过 HDWallet 可以派生出账户:

```js
const account = hdWallet.deriveAccount(2);//我们派生accountIndex=2的账户
```

当然，如果你有自己私钥，也可以通过指定私钥创建 Account：

```js
const account = Account.fromPrivateKey(
    '0x1000000000000000000000000000000000000000000000000000000000000000',
  );
```

当然，获取对应的公钥和地址也不在话下：

```js
const publicKey = account.publicKey;
const address = account.address;
```

到了这里，你已经成功创建了 Account，现在让我们进入 Client，来学习如何和链进行交互。

#### Step 4：构建 Client 对象，正式和链上的 Service 进行数据交互

如果想了解什么是 GraphQL，可以参考 [GraphQL 官方文档](https://graphql.org/)。
关于 Huobi Chain 的 GraphQL API 接口, 请参看接口章节。

GraphQL 将请求分类为 2 类：Query 和 Mutation。前者不会对数据进行任何形式的修改，是查。后者则相反，增珊改都可能发生。
Huobi Chain 的 GraphQL API接口也是如此。此外，Client类还提供了一些工具方法，这些方法不会发送请求到网络，所以他们不属于Huobi Chain GraphQL API接口，但是也被包含在Client类里。

目前的 API 大致分为如下：

**Query**
1. getBlock, getLatestBlockHeight and waitForNextNBlock
2. getTransaction
3. getReceipt
4. queryService and queryServiceDyn

**Mutation**
1. sendTransaction

**Locally**
1. composeTransaction

我们通过例子，一步一步来了解。因为Client必须知道通过那个接口和节点进行数据通信，所以必须提供uri。不过在本文档第一步构建 Muta 对象时，给出了 endpoint 参数，那么现在我们可以直接通过 muta 对象来获得一个 Client 对象：

```js
  let client = Muta.createDefaultMutaInstance().client();
```

当然,你也可以自己构建一个 Client 对象：

```js
  /**
   * or if you want to initialize a customized Client object, you could pass a ClientOption arg
   *
   * export interface ClientOption {
   *  endpoint: string; // you already know it
   *  chainId: string; //the Muta chain id, refer to >>genesis.toml<<
   *  maxTimeout: number; //this is the timeoutGap, please see ex1
   *  defaultCyclesLimit: Uint64; //below
   *  defaultCyclesPrice: Uint64; //below
   * }
   *
   */

  let client = new Client({
    chainId:
      '0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036',
    defaultCyclesLimit: '0xffff',
    defaultCyclesPrice: '0xffff',
    endpoint: 'http://127.0.0.1:8000/graphql',
    maxTimeout: DEFAULT_TIMEOUT_GAP * DEFAULT_CONSENSUS_INTERVAL,
  });
```

我们来解释下其中的参数：

你已经了解了endpoint，chainId。CyclesLimit 和 CyclesPrice 的概念类似于以太坊的 gasLimit 和 gasPrice。
defaultCyclesLimit 和 defaultCyclesPrice 是在将来发送 GraphQL API 请求时给定的默认值，当然你在发送请求的时候可以指定新的值。

maxTimeout = DEFAULT_TIMEOUT_GAP * DEFAULT_CONSENSUS_INTERVAL。
你已经了解了DEFAULT_TIMEOUT_GAP。因为区块链没有世界时钟，所以只能通过 block 高度 x 平均期望出块时间来大致计算出现实时间。Huobi Chain 内置 Overlord 共识算法的预期**单轮**出块时间是 3 秒，所以 DEFAULT_CONSENSUS_INTERVAL=3。


万事俱备，接下来我们开始与链进行交互。我们先尝试获得某个区块的信息，因为如果你能某一个区块的信息，就能获得所有的区块的信息，就能获得区块链的信息。

我们获得第 10 高度的区块：

```js
  const blockInfo = await client.getBlock('10');
```

也可以获得最新的高度的区块:

```js
  const latestBlockInfo = await client.getBlock(null);
```

当然，你可以直接获得最新区块的高度：

```js
  let latestBlockHeight = hexToNum(latestBlockInfo.header.height);
  // or more easy way
  latestBlockHeight = await client.getLatestBlockHeight();
```

接下来我们更进一步，我们从节点Query一些数据，还记得么 Query 和 Mutation 的差别么?

Huobi Chain 有很多 service，例如 metadata 服务会提供一些关于链的基础信息，asset 资产服务可以提供创建用户自定义 token的功能(User defined tokens)。服务之间通常居然有依赖关系，可以互相调用，构建出更高级的业务逻辑。如果你是要和内置服务交互，那么请参考我们的内置服务的 GraphQL API 接口手册，如果你是要和用户自定义服务交互，那么请咨询服务的开发团队。

为了进一步学习，我们现在向 AssetService 来发起 Query 请求，访问数据。在发起任何Query之前，我们都必须知道请求接口交互的数据格式是什么。假设我们要向 AssetService 来发起查询 Asset 的请求。那么查看 GraphQL API 接口手册，我们需要的数据类型是：

```typescript
type Hash = string;
export interface GetAssetParam {
  asset_id: Hash;
}
```

接口返回的数据类型是：

```typescript
type Hash = string;
type Address = string;
export interface Asset {
  asset_id: Hash;
  name: string;
  symbol: string;
  supply: number | BigNumber;
  issuer: Address;
}
```

其中 asset_id 是创建一个 Asset 后，Asset 服务返回的唯一标识。name 和 symbol 是用户自定义的标识，supply 是总量，issuer是创建账户。

现在我们通过 queryServiceDyn 方法来访问他，queryServiceDyn 和 queryService 的 api，请参考 SDK 文档或者 API 文档：

```typescript
  let asset : Asset | null = null;
  try {
    const asset_id =
      '0x0000000000000000000000000000000000000000000000000000000000000000';
    asset = await client.queryServiceDyn<
      GetAssetParam,
      Asset
      // tslint:disable-next-line:no-object-literal-type-assertion
    >({
      caller: '0x2000000000000000000000000000000000000000',
      method: 'get_balance',
      payload: { asset_id },
      serviceName: 'asset',
    } as ServicePayload<GetAssetParam>);
  } catch (e) {
    asset = null;
  }
```

很好，这段代码应该会进入 catch，然后设定 asset 为 null，毕竟我们什么 Asset 都没有创建过。这仅仅是一个Query，查询的例子。

现在我们进入增删改的部分，也就是 Mutation 请求。 SendTransaction 是一个 Mutation 的请求。那么我们来看看SendTransaction 需要提供那些数据。

```typescript
    public async sendTransaction(
    signedTransaction: SignedTransaction,
  ): Promise<Hash> 

    export interface SignedTransaction {
      chainId: string;
      cyclesLimit: string;
      cyclesPrice: string;
      nonce: string;
      timeout: string;
      serviceName: string;
      method: string;
      payload: string;
      txHash: string;
      pubkey: string;
      signature: string;
    }
```
可以看到，发送一笔交易，和大多数区块链类似，需要一笔被**签名**的交易

那么我们先来构建一笔**创建** Asset 交易，然后对其签名。

通过查询 GrapQL API 接口文档,

 - 创建Asset服务的服务名是: asset

 - 接口的方法为: create_asset,
 
 - 接受接受的参数为: CreateAssetParam

```typescript
    export interface CreateAssetParam {
      name: string;
      symbol: string;
      supply: number | BigNumber;
    }
```

那么我们通过 Client 的工具方法 composeTransaction 来构建一个这样的签名：

```typescript
    const tx = await client.composeTransaction<CreateAssetParam>({
        method: 'create_asset',
        payload: createAssetParam,
        serviceName: 'asset',
      });
```

随后我们需要使用一个用户，对其签名，那么这个用户就是这个 Asset 的 issuer. 还记得 Account 类型么？现在是他上场的时候了,使用你所期望的用户的 Account 对象调用 signTransaction 来对交易签名：

```typescript
    const signedTransaction = Account.fromPrivateKey(
        '0x1000000000000000000000000000000000000000000000000000000000000000',
      ).signTransaction(tx);
```

现在我们可以调用 signTransaction 来发送我们的交易了。和大多数区块链一样，由于是异步网络和起步业务系统，你所提交的交易可能不会被立刻提交到区块链上。发送交易后通常返回交易的位置标识哈希值。

```typescript
    const txHash = await client.sendTransaction(this.account.signTransaction(tx));
```

接下来我们只需要通过标识哈希定期去查询交易，看交易是否被成功提交到了区块链。如果一笔交易被成功地提交到了区块链，那么他将不可篡改不可回滚。

当区块链认为一笔交易比成功的提交了，他会返回一张 Receipt 交易凭证，给出了交易的诸多信息，以及交易执行后的返回，我们可以通过getReceipt 来获得凭证：

```typescript
    const receipt: Receipt = await this.client.getReceipt(utils.toHex(txHash));

```

Receipt凭证的数据类型如下:

```typescript
export interface Receipt {
  stateRoot: string; //交易被提交后的state的root
  height: string; //交易被提交进入的块的盖高度
  txHash: string; //该笔交易的唯一哈希表示
  cyclesUsed: string; //该笔交易使用的cycle
  events: Event[]; //该笔交易产生的事件
  response: ReceiptResponse; //该笔交易的返回
}

export interface ReceiptResponse {
  serviceName: string; //该笔交易调用的服务名称
  method: string; //该笔交易调用的服务方法
  ret: string; //服务给出的返回数据
  isError: boolean; //服务给出的返回结果,运行是否成功
}
```

请仔细阅读上面的数据结构，需要只出的是，ret 和 isError 可能同时给出。例如 ret 给出错误信息。返回 ret 数据是通用的字符串类型，但具体数据可是请参考对应服务的 GraphQL API 接口。

这里我们的 create_asset 方法返回的格式就是之前见过的 Asset 数据格式，并且是通过 JSON 来序列化的：

```typescript
export interface Asset {
  asset_id: Hash;
  name: string;
  symbol: string;
  supply: number | BigNumber;
  issuer: Address;
}
```

所以我们可以通过 JSON.parse 来把 ret 字符串转换成对应的 Asset 对象：

```typescript
  let createdAssetResult = utils.safeParseJSON(receipt.response.ret);//util工具类请参考API doc
```

#### Step5：通过使用 AssetService API，直接和 AssetService 交互

好的，通过Client的例子，你已经可以向任何服务发起数据交互了，但是每次都调用原生的 GraphQL API，非常的恼人，我相信你肯定可以把他们包装成对应的js方法。现在我们来看一看 Huobi Chain 内置的 AssetService 对应的 js-sdk：

老规矩，我们仍然需要一个 Client 对象和 Account 对象，就像上一节里我们用到的一样，作用也是一样的。随后我们创建一个AssetService：

```typescript
    const muta = Muta.createDefaultMutaInstance();
      const account = Account.fromPrivateKey(
        '0x1000000000000000000000000000000000000000000000000000000000000000',
      );
    
      /**
       * we build a service, pass the client and account object
       * nothing abnormal
       */
      const service = new AssetService(muta.client(), account);
```

接下来就非常简单了，我们直接创建一个资产，参数类型和之前的相同，不再赘述：

```typescript
  const createdAsset = await service.createAsset({
    name: 'LOVE_COIN',
    supply: 1314,
    symbol: 'LUV',
  });
  const assetId = createdAsset.asset_id;
```

我们通过 asset_id 来查询一个资产的状态：

```typescript
const asset = await service.getAsset(assetId);
```

查询一下某个用户的余额：

```typescript
const balance = await service.getBalance(assetId, '0x2000000000000000000000000000000000000000');
```

最后是向某个用户发送一定数量的 UDT，这里是 LOVE_COIN：

```typescript
  await service.transfer({
    asset_id: assetId,
    to:'0x2000000000000000000000000000000000000000',
    value: 520,
  });
```

好了！教程到此结束了，相信你已经可以熟练使用 JS-SDK 了。
