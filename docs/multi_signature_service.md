# Multi-signature Service

## 概述

Multi-signature Service 是 Muta 的原生多重签名实现。多重签名是一个地址由若干个账户所组成，每一个地址对应着一个权重。当签名的权重之和大于多签账户的阈值的时候，就代表一个有效的多重签名。多重签名账户的结构如下：

```rust
pub struct MultiSigPermission {
    pub owner:     Address,
    pub accounts:  Vec<Account>,
    pub threshold: u32,
    pub memo:      String,
}

pub struct Account {
    pub address:     Address,
    pub weight:      u8,
    pub is_multiple: bool,
}
```

`MultiSigPermission` 中的 owner 即是账户的管理员，memo 是多签账户的备注信息。Muta 原生多签允许多签嵌套，所以在 Account 结构中，用 is_multiple 表示是否是多签账户。Muta 多重签名有两点限制：

1. 多签嵌套不得超过 8 层。
2. 每一级账户的数量不得超过 16 个。

为了让交易的签名能够兼容多重签名，`SignedTransaction` 中 pubkey 是所有的签名者的公钥数组调用 `rlp::encode_list()` 序列化后的，signature 是签名数组调用 `rlp::encode_list()` 序列化后的结果。`RawTransaction` 中的 sender 是多签地址或者签名者的地址。

```rust
pub struct RawTransaction {
    pub chain_id:     Hash,
    pub cycles_price: u64,
    pub cycles_limit: u64,
    pub nonce:        Hash,
    pub request:      TransactionRequest,
    pub timeout:      u64,
    pub sender:       Address,
}

pub struct TransactionRequest {
    pub method:       String,
    pub service_name: String,
    pub payload:      JsonString,
}

pub struct SignedTransaction {
    pub raw:       RawTransaction,
    pub tx_hash:   Hash,
    pub pubkey:    Bytes,
    pub signature: Bytes,
}
```

## 接口

1. 创建多签账户

```rust
fn generate_account(&mut self, ctx: ServiceContext, payload: GenerateMultiSigAccountPayload) -> ServiceResponse<GenerateMultiSigAccountResponse>;

// 参数
pub struct GenerateMultiSigAccountPayload {
    pub owner:            Address,
    pub addr_with_weight: Vec<AddressWithWeight>,
    pub threshold:        u32,
    pub memo:             String,
}

pub struct AddressWithWeight {
    pub address: Address,
    pub weight:  u8,
}

pub struct GenerateMultiSigAccountResponse {
    pub address: Address,
}
```

GraphiQL 示例：

```graphql
mutation generate_account{
  unsafeSendTransaction(inputRaw: {
    serviceName:"multi_signature",
    method:"set_admin",
    payload:"{\"owner\":\"0x83dba2ed0eb6ffcc256250ef6e1b6d15b3f68e89\",\"addr_with_weight\":[{\"address\":\"0x639f87c481224a9ab7480828e86f76e39ca56865\",\"weight\":1},{\"address\":\"0x1459d1711eae368e6d685e5169b76e917202774f\",\"weight\":1},{\"address\":\"0x0e1655a22d6ca92928f5ce37c6b8b2845a0721bb\",\"weight\":1},{\"address\":\"0xe6533a1db7a4aad25922672eac1d22fa14e70fe8\",\"weight\":1}],\"threshold\":3,\"memo\":\"multi-signature-01\"}",
    timeout:"0x14",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x45c56be699dca666191ad3446897e0f480da234da896270202514a0e1a587c3f"
  )
}
```

2. 查询多签账户

```rust
fn get_account_from_address(&self, _ctx: ServiceContext, payload: GetMultiSigAccountPayload) -> ServiceResponse<GetMultiSigAccountResponse>;

// 参数
pub struct GetMultiSigAccountPayload {
    pub multi_sig_address: Address,
}


pub struct GetMultiSigAccountResponse {
    pub permission: MultiSigPermission,
}

pub struct MultiSigPermission {
    pub owner:     Address,
    pub accounts:  Vec<Account>,
    pub threshold: u32,
    pub memo:      String,
}
```

GraphiQL 示例：

```graphql
query get_account_from_address{
  queryService(
  caller: "0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1"
  serviceName: "multi_signature"
  method: "get_account_from_address"
  payload: "{\"multi_sig_address\":\"0xfe5a6c9959c49a6ef3546282318c17ce121db6ea\"}"
  ){
    ret,
    isError
  }
}
```

3. 创建更新多签账户

```rust
// 需要 admin 权限
fn update_account(&mut self, ctx: ServiceContext, payload: UpdateAccountPayload,) -> ServiceResponse<()>;

// 参数
pub struct UpdateAccountPayload {
    pub account_address:  Address,
    pub new_account_info: GenerateMultiSigAccountPayload,
}

pub struct GenerateMultiSigAccountPayload {
    pub owner:            Address,
    pub addr_with_weight: Vec<AddressWithWeight>,
    pub threshold:        u32,
    pub memo:             String,
}
```

GraphiQL 示例：

```graphql
mutation update_account{
  unsafeSendTransaction(inputRaw: {
    serviceName:"multi_signature",
    method:"set_admin",
    payload:"{\"account_address\":\"0x4537b38aeb100de3b9d4483c7f5e399e327a5246\",\"new_account_info\":{\"owner\":\"0x007c20181e1ad8f85027253d4afae34155ceaaa3\",\"addr_with_weight\":[{\"address\":\"0x13601c39ec94a3f88728403bff3d945006e2271f\",\"weight\":1},{\"address\":\"0xa4bf852474398106b121e90d40e2c27fd083b257\",\"weight\":1},{\"address\":\"0x4f5ae65ef07c14f0a13dfee4054c476cc1503517\",\"weight\":1},{\"address\":\"0x87ffd1bec3236c854f1523bb55427952fde4a48d\",\"weight\":1}],\"threshold\":3,\"memo\":\"multi-signature-01\"}}",
    timeout:"0x14",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x45c56be699dca666191ad3446897e0f480da234da896270202514a0e1a587c3f"
  )
}
```
