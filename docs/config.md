# 配置说明

默认的创世块和配置样例在 `config` 文件夹中，此处对其中的一些字段进行说明。

## `genesis.toml`

创世块

```toml
timestamp = 0
prevhash = "44915be5b6c20b0678cf05fcddbbaa832e25d7e6ac538784cd5c24de00d47472"

# 各个 service 的初始化参数
[[services]]
name = "asset"
payload = '''
{
    "id": "f56924db538e77bb5951eb5ff0d02b88983c49c45eea30e8ae3e7234b311436c",
    "name": "Huobi Token",
    "symbol": "HT",
    "supply": 1000000000,
    "issuer": "f8389d774afdad8755ef8e629e5a154fddc6325a"
}
'''

[[services]]
name = "metadata"
payload = '''
{
    "chain_id": "b6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    "common_ref": "703873635a6b51513451",
    "timeout_gap": 20,
    "cycles_limit": 999999999999,
    "cycles_price": 1,
    "interval": 3000,
    "verifier_list": [
        {
            "address": "f8389d774afdad8755ef8e629e5a154fddc6325a",
            "propose_weight": 1,
            "vote_weight": 1
        }
    ],
    "propose_ratio": 15,
    "prevote_ratio": 10,
    "precommit_ratio": 10
}
'''

[[services]]
name = "node_manager"
# private key of this admin:
# 2b672bb959fa7a852d7259b129b65aee9c83b39f427d6f7bded1f58c4c9310c2
payload = '{"admin": "0xcff1002107105460941f797828f468667aa1a2db"}'
```

metadata 部分初始化字段说明：
- chain_id: 链唯一 id
- common_ref: bls 签名需要
- cycles_limit: 交易最大 cycle 限制
- interval: 出块间隔，单位为 ms
- verifier_list: 共识列表

## `chain.toml`

链的运行配置：

```toml
# 节点私钥，节点的唯一标识，在作为 bootstraps 节点时，需要给出地址和该私钥对应的公钥让其他节点连接；如果是出块节点，该私钥对应的地址需要在 consensus verifier_list 中
privkey = "45c56be699dca666191ad3446897e0f480da234da896270202514a0e1a587c3f"

# db config，链数据所在目录
data_path = "./data"

[graphql]
# graphql 监听地址
listening_address = "0.0.0.0:8000"
graphql_uri = "/graphql"
graphiql_uri = "/graphiql"

[network]
# 链 p2p 网络监听地址
listening_address = "0.0.0.0:1337"
rpc_timeout = 10

# 起链时连接的初始节点信息
[[network.bootstraps]]
pubkey = "031288a6788678c25952eba8693b2f278f66e2187004b64ac09416d07f83f96d5b"
address = "0.0.0.0:1888"

[mempool]
# 交易池大小
pool_size = 20000
# 一次批量广播的交易数量
broadcast_txs_size = 200
# 交易广播间隔
broadcast_txs_interval = 200

[consensus]
# 共识节点的 bls 公钥，各节点的该配置需完全一致
public_keys = [ "04188ef9488c19458a963cc57b567adde7db8f8b6bec392d5cb7b67b0abc1ed6cd966edc451f6ac2ef38079460eb965e890d1f576e4039a20467820237cda753f07a8b8febae1ec052190973a1bcf00690ea8fc0168b3fbbccd1c4e402eda5ef22" ]

[executor]
# 设为 true 时，节点将只保存最新高度的 state
light = false

[logger]
filter = "info"
log_to_console = true
console_show_file_and_line = false
log_path = "logs/"
log_to_file = true
metrics = true
# you can specify log level for modules with config below
# modules_level = { "overlord::state::process" = "debug", core_consensus = "error" }
```