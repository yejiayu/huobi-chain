# FAQ

#### Overlord 共识算法与 Tendermint 等共识算法相比有哪些改进？
<div align=center><img src="./static/overlord_compare.png"></div>

#### 为什么 block 1 的 `proof.blockHash` 和 `prevHash` 不一致？

在链成功启动后，你可能会发现在链的 block 1（创世块后是 block 1）中， `proof.blockHash` 和 `prevHash` 是不一致的。为什么呢？其实在链的整个共识过程中，`prevHash` 和 `proof.blockhash` 都是上一个 block 序列化之后算出的 hash，两者应该是一致的。但是由于创世块的共识不是来源于共识机制，而是来源于社区共识，而第一个块的 proof 是对创世块的共识的证明，所以针对创世块的这种特殊性，我们对第一个块中的 `proof.blockhash` 做了特殊处理：高度 0 是创世块，我们从高度 1 开始共识，高度 1 的 block 中的 `prevHash` 是创世块序列化之后的 hash，`proof.blockhash` 是空 hash。所以才有了这样的不一致，这是正常的。

#### `interval` （出块间隔，单位为 ms）设置为 3000 时，实际出块时间却不是严格的 3s？

这个是正常现象。这是因为在共识过程中，我们让出块节点的 `interval` 保持在 3s，非出块节点则没有限制，因此实际出块时间并不是严格的 3s。通常情况下，如果网络条件较好，实际出块时间可能在 1.5s~3s 之前波动，如果网络条件较差，实际出块时间可能会略高于 3s。

#### 从 release 页面下载的 binary，跑在 ubuntu16 上的时候，会报 `huobi-chain: error while loading shared libraries: libssl.so.1.1: cannot open shared object file: No such file or directory`。

Huobi-chain 编译时会自动检测系统的 OpenSSL 版本，在运行时进行动态链接。

release 页面的 Linux binary 是使用 [Travis](https://github.com/HuobiGroup/huobi-chain/blob/master/.travis.yml#L42) 自动编译的，其默认使用的虚拟机版本为 [Ubuntu 18](https://docs.travis-ci.com/user/reference/overview/#for-a-particular-travisyml-configuration)。

Ubuntu 18 和 16 的默认 OpenSSL 版本不一致，所以运行时，动态链接会报错。

遇到该问题时，可以使用在 Ubuntu 16 环境下手动编译的 binary。其它系统（如 CentOS 7）下，可类似解决。
