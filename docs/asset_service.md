# Asset Service

## 概述

Asset service 是 Huobi Chain 的内置资产模块，负责管理链原生资产以及第三方发行资产。

- 资产成为一等公民：加密资产作为区块链的核心，理应成为一等公民。
Asset 模块利用 Muta 框架提供的 service 能力，为所有资产提供链级别的支持，为面向资产编程提供支持。
  
- 第三方发行资产： 用户可以使用 Asset 模块发行资产，自定义资产属性和总量等

- 资产与合约交互： 未来可以打通虚拟机和资产模块，为资产的广泛使用提供支持

## 接口

Asset 模块采用类似以太坊 ERC-20 的接口设计，主要包含：

1. 发行资产

```rust
// 资产数据结构
pub struct Asset {
    pub id:     Hash,
    pub name:   String,
    pub symbol: String,
    pub supply: u64, // 以最小单位表示的发行量。比如，supply 为 100，precision 为 1 时，表示发行 10 个 token
    pub precision: u64, // 资产数值精度，1 表示 10 的一次方，以此类推
    pub issuer: Vec<Address>, // 资产发行者的地址
    pub relayable: bool, // 资产是否可转移到其他链（如 Ethereum）上
}

// 发行资产接口
// 资产 ID 自动生成，确保唯一
fn create_asset(&mut self, ctx: ServiceContext, payload: CreateAssetPayload) -> ServiceResponse<Asset>;

// 发行资产参数
pub struct CreateAssetPayload {
    pub name:   String,
    pub symbol: String,
    pub supply: u64,
    pub precision: u64,
    pub relayable: bool,
}
```

GraphiQL 示例：

```graphql
mutation create_asset{
  unsafeSendTransaction(inputRaw: {
    serviceName:"asset",
    method:"create_asset",
    payload:"{\"name\":\"Test Coin\",\"symbol\":\"TC\",\"supply\":100000000, \"precision\":2, \"relayable\": false}",
    timeout:"0x172",
    nonce:"0x9db2d7efe2b61a88827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999"
  }, inputPrivkey: "0x30269d47fcf602b889243722b666881bf953f1213228363d34cf04ddcd51dfd2"
  )
}
```

2. 查询资产信息

```rust
// 查询接口
fn get_asset(&self, ctx: ServiceContext, payload: GetAssetPayload) -> ServiceResponse<Asset>；

// 查询参数
pub struct GetAssetPayload {
    pub id: Hash, // 资产 ID
}
```

GraphiQL 示例：

```graphql 
query get_asset{
  queryService(
  caller: "0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1"
  serviceName: "asset"
  method: "get_asset"
  payload: "{\"id\": \"0x5f1364a8e6230f68ccc18bc9d1000cedd522d6d63cef06d0062f832bdbe1a78a\"}"
  ){
    ret,
    isError
  }
}
```

3. 转账

```rust
// 转账接口
fn transfer(&mut self, ctx: ServiceContext, payload: TransferPayload) -> ServiceResponse<()>；

// 转账参数
pub struct TransferPayload {
    pub asset_id: Hash,
    pub to:       Address,
    pub value:    u64,
    pub memo:     String,
}
```

GraphiQL 示例：

```graphql
mutation transfer{
  unsafeSendTransaction(inputRaw: {
    serviceName:"asset",
    method:"transfer",
    payload:"{\"asset_id\":\"0x5f1364a8e6230f68ccc18bc9d1000cedd522d6d63cef06d0062f832bdbe1a78a\",\"to\":\"0xf8389d774afdad8755ef8e629e5a154fddc6325a\", \"value\":10000, \"memo\":\"transfer to David\"}",
    timeout:"0x289",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x30269d47fcf602b889243722b666881bf953f1213228363d34cf04ddcd51dfd2"
  )
}
```

4. 查询余额

```rust
// 查询接口
fn get_balance(&self, ctx: ServiceContext, payload: GetBalancePayload) -> ServiceResponse<GetBalanceResponse>;

// 查询参数
pub struct GetBalancePayload {
    pub asset_id: Hash,
    pub user:     Address,
}

// 返回值
pub struct GetBalanceResponse {
    pub asset_id: Hash,
    pub user:     Address,
    pub balance:  u64,
}
```

GraphiQL 示例： 

```graphql
query get_balance{
  queryService(
  caller: "0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1"
  serviceName: "asset"
  method: "get_balance"
  payload: "{\"asset_id\": \"0x5f1364a8e6230f68ccc18bc9d1000cedd522d6d63cef06d0062f832bdbe1a78a\", \"user\": \"0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1\"}"
  ){
    ret,
    isError
  }
}
```

5. 批准额度

```rust
// 批准接口
fn approve(&mut self, ctx: ServiceContext, payload: ApprovePayload) -> ServiceResponse<()>;

// 批准参数
pub struct ApprovePayload {
    pub asset_id: Hash,
    pub to:       Address,
    pub value:    u64,
    pub memo:     String,
}
```

GraphiQL 示例： 

```graphql
  unsafeSendTransaction(inputRaw: {
    serviceName:"asset",
    method:"approve",
    payload:"{\"asset_id\":\"0x5f1364a8e6230f68ccc18bc9d1000cedd522d6d63cef06d0062f832bdbe1a78a\",\"to\":\"0xf8389d774afdad8755ef8e629e5a154fddc6325a\", \"value\":10000, \"memo\":\"approve to Gum\"}",
    timeout:"0x378",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x30269d47fcf602b889243722b666881bf953f1213228363d34cf04ddcd51dfd2"
  )
}
```

6. 授权转账

```rust
// 接口
fn transfer_from(&mut self, ctx: ServiceContext, payload: TransferFromPayload) -> ServiceResponse<()>；

// 参数
pub struct TransferFromPayload {
    pub asset_id:  Hash,
    pub sender:    Address,
    pub recipient: Address,
    pub value:     u64,
    pub memo:      String,
}
```

GraphiQL 示例：

```graphql
mutation transfer_from{
  unsafeSendTransaction(inputRaw: {
    serviceName:"asset",
    method:"transfer_from",
    payload:"{\"asset_id\":\"0x5f1364a8e6230f68ccc18bc9d1000cedd522d6d63cef06d0062f832bdbe1a78a\",\"sender\":\"0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1\", \"recipient\":\"0xfffffd774afdad8755ef8e629e5a154fddc6325a\", \"value\":5000, \"memo\":\"transfer from Alice to Bob\"}",
    timeout:"0x12c",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x45c56be699dca666191ad3446897e0f480da234da896270202514a0e1a587c3f"
  )
}
```

7. 查询限额

```rust
// 查询接口
fn get_allowance(&self, ctx: ServiceContext, payload: GetAllowancePayload) -> ServiceResponse<GetAllowanceResponse>；

// 查询参数
pub struct GetAllowancePayload {
    pub asset_id: Hash,
    pub grantor:  Address,
    pub grantee:  Address,
}

// 返回值
pub struct GetAllowanceResponse {
    pub asset_id: Hash,
    pub grantor:  Address,
    pub grantee:  Address,
    pub value:    u64,
}
```

GraphiQL 示例：

```graphql
query get_allowance{
  queryService(
  caller: "0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1"
  serviceName: "asset"
  method: "get_allowance"
  payload: "{\"asset_id\": \"0x5f1364a8e6230f68ccc18bc9d1000cedd522d6d63cef06d0062f832bdbe1a78a\", \"grantor\": \"0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1\", \"grantee\": \"0xf8389d774afdad8755ef8e629e5a154fddc6325a\"}"
  ){
    ret,
    isError
  }
}
```

8. 查询链的原生资产

链的原生资产为创世启动时发行的资产，资产信息写在 `Asset Service` 创世配置文件中。

```rust
// 查询接口：返回原生资产
fn get_native_asset(&self, ctx: ServiceContext) -> ServiceResponse<Asset>;
```

GraphiQL 示例：

```graphql
query get_native_asset{
  queryService(
  caller: "0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1"
  serviceName: "asset"
  method: "get_native_asset"
  payload: ""
  ){
    ret,
    isError
  }
}
```

9. 修改 asset 模块的管理员

```rust
// 接口
fn change_admin(&mut self, ctx: ServiceContext, payload: ChangeAdminPayload) -> ServiceResponse<()>；

// 参数
pub struct ChangeAdminPayload {
    pub addr: Address,
}
```

GraphiQL 示例：

```graphql
mutation change_admin{
  unsafeSendTransaction(inputRaw: {
    serviceName:"asset",
    method:"change_admin",
    payload:"{\"addr\":\"0x5f1364a8e6230f68ccc18bc9d1000cedd522d6d63cef06d0062f832bdbe1a78a\"}",
    timeout:"0x12c",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x45c56be699dca666191ad3446897e0f480da234da896270202514a0e1a587c3f"
  )
}
```

10. 铸币

```rust
// 接口
fn mint(&mut self, ctx: ServiceContext, payload: MintAssetPayload) -> ServiceResponse<()>；

// 参数
pub struct MintAssetPayload {
    pub asset_id: Hash,
    pub to:       Address,
    pub amount:   u64,
    pub proof:    Hex,  // 铸币证明，目前作为保留字段
    pub memo:     String,
}
```

GraphiQL 示例：

```graphql
mutation mint{
  unsafeSendTransaction(inputRaw: {
    serviceName:"asset",
    method:"mint",
    payload:"{\"asset_id\":\"0x5f1364a8e6230f68ccc18bc9d1000cedd522d6d63cef06d0062f832bdbe1a78a\",\"to\":\"0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1\", \"amount\":5000, \"proof\":\"\", \"memo\":\"transfer from Alice to Bob\"}",
    timeout:"0x12c",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x45c56be699dca666191ad3446897e0f480da234da896270202514a0e1a587c3f"
  )
}
```

11. 销毁资产

```rust
// 接口
fn burn(&mut self, ctx: ServiceContext, payload: BurnAssetPayload) -> ServiceResponse<()>；

// 参数
pub struct MintAssetPayload {
    pub asset_id: Hash,
    pub amount:   u64,
    pub proof:    Hex,  // 销毁证明，目前作为保留字段
    pub memo:     String,
}
```

GraphiQL 示例：

```graphql
mutation burn{
  unsafeSendTransaction(inputRaw: {
    serviceName:"asset",
    method:"burn",
    payload:"{\"asset_id\":\"0x5f1364a8e6230f68ccc18bc9d1000cedd522d6d63cef06d0062f832bdbe1a78a\",\"amount\":5000, \"proof\":\"\", \"memo\":\"transfer from Alice to Bob\"}",
    timeout:"0x12c",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x45c56be699dca666191ad3446897e0f480da234da896270202514a0e1a587c3f"
  )
}
```

12. 转移资产

```rust
// 接口
fn relay(&mut self, ctx: ServiceContext, payload: RelayAssetPayload) -> ServiceResponse<()>；

// 参数
pub struct RelayAssetPayload {
    pub asset_id: Hash,
    pub amount:   u64,
    pub proof:    Hex,  // 转移资产证明，目前作为保留字段
    pub memo:     String,
}
```

GraphiQL 示例：

```graphql
mutation relay{
  unsafeSendTransaction(inputRaw: {
    serviceName:"asset",
    method:"relay",
    payload:"{\"asset_id\":\"0x5f1364a8e6230f68ccc18bc9d1000cedd522d6d63cef06d0062f832bdbe1a78a\",\"amount\":5000, \"proof\":\"\", \"memo\":\"transfer from Alice to Bob\"}",
    timeout:"0x12c",
    nonce:"0x9db2d7efe2b61a28827e4836e2775d913a442ed2f9096ca1233e479607c27cf7",
    chainId:"0xb6a4d7da21443f5e816e8700eea87610e6d769657d6b8ec73028457bf2ca4036",
    cyclesPrice:"0x9999",
    cyclesLimit:"0x9999",
    }, inputPrivkey: "0x45c56be699dca666191ad3446897e0f480da234da896270202514a0e1a587c3f"
  )
}
```
