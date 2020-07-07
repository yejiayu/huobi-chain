# Authorization Service

# 概述

Authorization Service 是被交易池调用，对交易进行检查的 service。Authorization Service 本身不提供任何的检查逻辑，而是去调用注册在其中的 service 及其对应的方法进行校验。

## 具体设计

Authorization Service 的结构如下所示：

```rust
pub struct AuthorizationService<SDK> {
    sdk:          SDK,
    verified_list: Box<dyn StoreArray<String, String>>,
}
```

从结构中可以看出，Authorization Service 主要是维护了一个 `verifiied_list`。`verified_list` 是一个（service 名字 — 方法名字）的数组。
<div align=center><img src="./static/auth.svg"></div>
如上图所示，当交易池收到交易的时候，交易池会调用 Authorization Service，将交易序列化之后发给 Authorization Service，这时 Authorization Service 会首先调用 multi-signature service 的 `verify_signature` 方法对交易进行验签，接着根据 `verified_list` 中注册的内容依次调用其他 service 中的方法对交易进行校验。因为 Authorization Service 并不清楚其他 service 的实现，所以注册进来的方法的 payload 都只能接收 `SignedTransaction` 序列化成的 Json 字符串作为参数，而返回值必须是 `ServiceResponse<()>` 当返回结果的调用 `is_error()` 方法返回真的时候，则判定交易不合法。注册进 Authorization Service 的方法必须遵循上述规范。

**注意：**验签方法已经预先在 Authorization Service 中注册，无需手动注册。

在 Authorization Service 中注册检查项有两种方法：

1. 在创世块中注册

如下代码中是在创世块中注册检查项的例子：

```toml
[[services]]
name = "authorization"
payload = '''
{
    "admin": "0xcff1002107105460941f797828f468667aa1a2db",
    "register_service_names": [
        "admission_control",
        "admission_control"
    ],
    "verified_method_names": [
        "is_permitted"
        "is_valid"
    ]
}
'''
```

上述代码中在 Authorization Service 中注册了两个检查项，分别是 admission_control service 中的 `is_permitted` 方法和 admission_control service 中的 `is_valid` 方法。

2. 通过 API 增删检查项

```rust
// 皆要求是 admin
fn add_verified_item(&mut self, ctx: ServiceContext, payload: AddVerifiedItemPayload) -> ServiceResponse<()> {}

fn remove_verified_item(&mut self, ctx: ServiceContext, payload: RemoveVerifiedItemPayload) -> ServiceResponse<()> {}

// 参数
pub struct AddVerifiedItemPayload {
    pub service_name: String,
    pub method_name:  String,
}

pub struct RemoveVerifiedItemPayload {
    pub service_name: String,
}
```

```graphql
query add_verified_item{
  queryService(
  caller: "0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1"
  serviceName: "authorization"
  method: "add_verified_item"
  payload: "{\"service_name\":\"multi-signature\",\"method_name\":\"verify_signature\"}"
  ){
    ret,
    isError
  }
}

query remove_verified_item{
  queryService(
  caller: "0x016cbd9ee47a255a6f68882918dcdd9e14e6bee1"
  serviceName: "authorization"
  method: "remove_verified_item"
  payload: "{\"service_name\":\"multi-signature\",\"method_name\":\"verify_signature\"}"
  ){
    ret,
    isError
  }
}
```
