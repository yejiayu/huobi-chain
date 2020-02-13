# Node Manager Service
Node Manager Service 负责变更节点的共识配置，并对变更权限进行管理。这些信息存储在 Metadata Service 中，在 `Metadata` 中可以动态变更的字段有  `interval`、`verifier_list`、 `propose_ratio`、 `prevote_ratio`、`precommit_ratio`、`brake_ratio` 。只有 admin 账户有权限进行变更操作，admin 账户的初始值写在 `config/genesis.toml` 配置文件中，起链后可以发交易给 Node Manager Service 进行修改。

## 接口

1. 读取 admin 地址
   
```rust
fn get_admin(&self, ctx: ServiceContext) -> ProtocolResult<Address>；

// Example: graphiql send tx
query get_admin{
  queryService(
  caller: "016cbd9ee47a255a6f68882918dcdd9e14e6bee1"
  serviceName: "node_manager"
  method: "get_admin"
  payload: ""
  ){
    ret,
    isError
  }
}
```

2. 设置 admin 地址
   
```rust
// 需要 admin 权限
fn set_admin(&mut self, ctx: ServiceContext, payload: SetAdminPayload) -> ProtocolResult<()>；

// 参数
pub struct SetAdminPayload {
    pub admin: Address,
}

// Example: graphiql send tx
mutation set_admin{
  unsafeSendTransaction(inputRaw: {
    serviceName:"node_manager",
    method:"set_admin",
    payload:"{\"admin\": \"016cbd9ee47a255a6f68882918dcdd9e14e6bee1\"}",
    timeout:"0x14",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"b6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x45c56be699dca666191ad3446897e0f480da234da896270202514a0e1a587c3f"
  )
}
```

3. 更新元数据
   
```rust
// 需要 admin 权限
fn update_metadata(&mut self, ctx: ServiceContext, payload: UpdateMetadataPayload) -> ProtocolResult<()>；

// 参数
pub struct UpdateMetadataPayload {
    pub verifier_list:   Vec<ValidatorExtend>,
    pub interval:        u64,
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio:  u64
}

pub struct ValidatorExtend {
    pub bls_pub_key: String,
    pub address:        Address,
    pub propose_weight: u32,
    pub vote_weight:    u32,
}

// Example: graphiql send tx
mutation update_metadata{
  unsafeSendTransaction(inputRaw: {
    serviceName:"node_manager",
    method:"update_metadata",
    payload:"{\"verifier_list\": [{\"bls_pub_key\": \"04188ef9488c19458a963cc57b567adde7db8f8b6bec392d5cb7b67b0abc1ed6cd966edc451f6ac2ef38079460eb965e890d1f576e4039a20467820237cda753f07a8b8febae1ec052190973a1bcf00690ea8fc0168b3fbbccd1c4e402eda5ef22\", \"address\": \"f8389d774afdad8755ef8e629e5a154fddc6325a\", \"propose_weight\": 5, \"vote_weight\": 5}], \"interval\": 5000, \"propose_ratio\": 5, \"prevote_ratio\": 5, \"precommit_ratio\": 5, \"brake_ratio\": 5}",
    timeout:"0xbe",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"b6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x30269d47fcf602b889243722b666881bf953f1213228363d34cf04ddcd51dfd2"
  )
}
```

5. 更新区块间隔
   
```rust
// 需要 admin 权限
fn update_interval(&mut self, ctx: ServiceContext, payload: UpdateIntervalPayload) -> ProtocolResult<()>；

// 参数
pub struct UpdateIntervalPayload {
    pub interval: u64,
}

// Example: graphiql send tx
mutation update_interval{
  unsafeSendTransaction(inputRaw: {
    serviceName:"node_manager",
    method:"update_interval",
    payload:"{\"interval\": 666}",
    timeout:"0x20",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"b6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x45c56be699dca666191ad3446897e0f480da234da896270202514a0e1a587c3f"
  )
}
```

6. 更新验证人集合
   
```rust
// 需要 admin 权限
fn update_validators(&mut self, ctx: ServiceContext, payload: UpdateValidatorsPayload) -> ProtocolResult<()>；

// 参数
pub struct UpdateValidatorsPayload {
    pub verifier_list: Vec<ValidatorExtend>,
}

pub struct ValidatorExtend {
    pub bls_pub_key: String,
    pub address:        Address,
    pub propose_weight: u32,
    pub vote_weight:    u32,
}

// Example: graphiql send tx
mutation update_validators{
  unsafeSendTransaction(inputRaw: {
    serviceName:"node_manager",
    method:"update_validators",
    payload:"{\"verifier_list\": [{\"bls_pub_key\": \"04188ef9488c19458a963cc57b567adde7db8f8b6bec392d5cb7b67b0abc1ed6cd966edc451f6ac2ef38079460eb965e890d1f576e4039a20467820237cda753f07a8b8febae1ec052190973a1bcf00690ea8fc0168b3fbbccd1c4e402eda5ef22\", \"address\": \"016cbd9ee47a255a6f68882918dcdd9e14e6bee1\", \"propose_weight\": 5, \"vote_weight\": 5}]}",
    timeout:"0xbe",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"b6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x30269d47fcf602b889243722b666881bf953f1213228363d34cf04ddcd51dfd2"
  )
}
```

7. 更新共识 round 超时时间
   
```rust
// 需要 admin 权限
fn update_ratio(&mut self, ctx: ServiceContext, payload: UpdateRatioPayload) -> ProtocolResult<()>；

// 参数
pub struct UpdateRatioPayload {
    pub propose_ratio:   u64,
    pub prevote_ratio:   u64,
    pub precommit_ratio: u64,
    pub brake_ratio: u64
}

// Example: graphiql send tx
mutation update_ratio{
  unsafeSendTransaction(inputRaw: {
    serviceName:"node_manager",
    method:"update_ratio",
    payload:"{\"propose_ratio\": 5, \"prevote_ratio\": 5, \"precommit_ratio\": 5, \"brake_ratio\": 5}",
    timeout:"0xbe",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"b6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x30269d47fcf602b889243722b666881bf953f1213228363d34cf04ddcd51dfd2"
  )
}
```
