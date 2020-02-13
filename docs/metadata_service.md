# Metadata Service
Metadata Service 负责存储链的元数据信息，包括：

```rust
pub struct Metadata {
    pub chain_id:        Hash,
    pub common_ref:      String, // BLS 签名算法的公共参数
    pub timeout_gap:     u64, // (交易有效期 - 当前区块高度)的最大值
    pub cycles_limit:    u64, // 区块全部交易消耗的 cycles 上限
    pub cycles_price:    u64, // 节点设置的交易打包进区块的最小 cycles_price
    pub interval:        u64, // 区块产生间隔
    pub verifier_list:   Vec<ValidatorExtend>, // 共识验证人列表
    pub propose_ratio:   u64, // 共识 propose 阶段的超时时间与 interval 的比值
    pub prevote_ratio:   u64, // 共识 prevote 阶段的超时时间与 interval 的比值
    pub precommit_ratio: u64, // 共识 precommit 阶段的超时时间与 interval 的比值
    pub brake_ratio: u64 // 共识重发 choke 的超时时间与 interval 的比值
}

pub struct ValidatorExtend {
    pub bls_pub_key: String,
    pub address:        Address,
    pub propose_weight: u32, //出块权重
    pub vote_weight:    u32, // 投票权重
}
```
通过 Metadata Service 可以读取这些信息，接口如下： 

## 接口
### 读取链元数据信息
   
```rust
fn get_metadata(&self, ctx: ServiceContext) -> ProtocolResult<Metadata>；

// Example: graphiql send tx
query get_admin{
  queryService(
  caller: "016cbd9ee47a255a6f68882918dcdd9e14e6bee1"
  serviceName: "metadata"
  method: "get_metadata"
  payload: ""
  ){
    ret,
    isError
  }
}
```
