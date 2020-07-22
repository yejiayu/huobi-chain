# Metadata Service

## 概述

Metadata Service 负责存储链的元数据信息，包括：

```rust
pub struct Metadata {
    pub chain_id:        Hash,
    pub common_ref:      Hex, // BLS 签名算法的公共参数
    pub timeout_gap:     u64, // (交易有效期 - 当前区块高度)的最大值
    pub cycles_limit:    u64, // 区块全部交易消耗的 cycles 上限
    pub cycles_price:    u64, // 节点设置的交易打包进区块的最小 cycles_price
    pub interval:        u64, // 区块产生间隔
    pub verifier_list:   Vec<ValidatorExtend>, // 共识验证人列表
    pub propose_ratio:   u64, // 共识 propose 阶段的超时时间与 interval 的比值
    pub prevote_ratio:   u64, // 共识 prevote 阶段的超时时间与 interval 的比值
    pub precommit_ratio: u64, // 共识 precommit 阶段的超时时间与 interval 的比值
    pub brake_ratio: u64,     // 共识重发 choke 的超时时间与 interval 的比值
    pub tx_num_limit:    u64, // 一个区块允许的最大交易数量
    pub max_tx_size:     u64, // 允许的交易最大字节大小
}

pub struct ValidatorExtend {
    pub bls_pub_key: Hex,
    pub address:        Address,
    pub propose_weight: u32, //出块权重
    pub vote_weight:    u32, // 投票权重
}
```
通过 Metadata Service 可以读取这些信息，接口如下： 

## 接口
1. 读取链元数据信息
   
```rust
// 接口
fn get_metadata(&self, ctx: ServiceContext) -> ServiceResponse<Metadata>；
```

GraphiQL 示例：

```graphql
query get_metadata{
  queryService(
  caller: "0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1"
  serviceName: "metadata"
  method: "get_metadata"
  payload: ""
  ){
    ret,
    isError
  }
}
```

2. 更新链元数据
   
```rust
// 接口
fn update_metadata(&self, ctx: ServiceContext, payload: UpdateMetadataPayload) -> ServiceResponse<()>；

// 参数
pub struct UpdateMetadataPayload {
    pub verifier_list:   Vec<ValidatorExtend>,
    pub interval:        u64,
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:     u64,
    pub timeout_gap:     u64,
    pub cycles_limit:    u64,
    pub cycles_price:    u64,
    pub tx_num_limit:    u64,
    pub max_tx_size:     u64,
}
```

GraphiQL 示例：

```graphql
mutation update_metadata{
  unsafeSendTransaction(inputRaw: {
    serviceName:"metadata",
    method:"update_metadata",
    payload:"{\"timeout_gap\": 99999, \"cycles_limit\": 999999999999, \"cycles_price\": 1, \"interval\": 3000, \"verifier_list\": [{\"bls_pub_key\": \"0x04188ef9488c19458a963cc57b567adde7db8f8b6bec392d5cb7b67b0abc1ed6cd966edc451f6ac2ef38079460eb965e890d1f576e4039a20467820237cda753f07a8b8febae1ec052190973a1bcf00690ea8fc0168b3fbbccd1c4e402eda5ef22\", \"address\": \"0xf8389d774afdad8755ef8e629e5a154fddc6325a\", \"propose_weight\": 1, \"vote_weight\": 1}], \"propose_ratio\": 15, \"prevote_ratio\": 10, \"precommit_ratio\": 10, \"brake_ratio\": 7, \"tx_num_limit\": 9000, \"max_tx_size\": 10485760}",
    timeout:"0x289",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x30269d47fcf602b889243722b666881bf953f1213228363d34cf04ddcd51dfd2"
  )
}
```
