#[cfg(test)]
mod tests;
pub mod types;

use std::ops::{Deref, DerefMut};

use bytes::Bytes;
use derive_more::Display;
use serde::Serialize;

use binding_macro::{cycles, genesis, service, tx_hook_before, write};
use protocol::traits::{ExecutorParams, ServiceResponse, ServiceSDK, StoreMap, StoreUint64};
use protocol::types::{Address, Hash, ServiceContext};

use crate::types::{
    ApproveEvent, ApprovePayload, Asset, AssetBalance, CreateAssetPayload, GetAllowancePayload,
    GetAllowanceResponse, GetAssetPayload, GetBalancePayload, GetBalanceResponse,
    InitGenesisPayload, TransferEvent, TransferFromEvent, TransferFromPayload, TransferPayload,
};

const NATIVE_ASSET_KEY: &str = "native_asset";
const FEE_ASSET_KEY: &str = "fee_asset";
const FEE_ACCOUNT_KEY: &str = "fee_account";

pub struct AssetService<SDK> {
    sdk:    SDK,
    assets: Box<dyn StoreMap<Hash, Asset>>,
    fee:    Box<dyn StoreUint64>,
}

impl<SDK: ServiceSDK> Deref for AssetService<SDK> {
    type Target = SDK;

    fn deref(&self) -> &Self::Target {
        &self.sdk
    }
}

impl<SDK: ServiceSDK> DerefMut for AssetService<SDK> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.sdk
    }
}

#[service]
impl<SDK: ServiceSDK> AssetService<SDK> {
    pub fn new(mut sdk: SDK) -> Self {
        let assets: Box<dyn StoreMap<Hash, Asset>> = sdk.alloc_or_recover_map("assets");
        let fee = sdk.alloc_or_recover_uint64("fee");

        Self { sdk, assets, fee }
    }

    #[genesis]
    fn init_genesis(&mut self, payload: InitGenesisPayload) {
        let asset = Asset {
            id:        payload.id.clone(),
            name:      payload.name,
            symbol:    payload.symbol,
            supply:    payload.supply,
            precision: payload.precision,
            issuer:    payload.issuer.clone(),
        };
        self.assets.insert(asset.id.clone(), asset.clone());

        self.set_value(NATIVE_ASSET_KEY.to_owned(), payload.id.clone());
        self.set_value(FEE_ASSET_KEY.to_owned(), payload.id);
        self.set_value(FEE_ACCOUNT_KEY.to_owned(), payload.fee_account);
        self.fee.set(payload.fee);
        self.set_account_value(&asset.issuer, asset.id, AssetBalance::new(payload.supply))
    }

    #[tx_hook_before]
    fn tx_hook_before_(&mut self, ctx: ServiceContext) {
        let caller = ctx.get_caller();

        let fee_acct: Address = self
            .get_value(&FEE_ACCOUNT_KEY.to_owned())
            .expect("fee asset should not be empty");
        if caller == fee_acct {
            return;
        }

        let value = self.fee.get();
        let asset_id: Hash = self
            .get_value(&FEE_ASSET_KEY.to_owned())
            .expect("fee account should not be empty");

        // FIXME: Refactor hook api
        let _ = self._transfer(&caller, &fee_acct, asset_id, value);
    }

    #[cycles(100_00)]
    #[read]
    fn get_native_asset(&self, ctx: ServiceContext) -> ServiceResponse<Asset> {
        let asset_id: Hash = self
            .get_value(&NATIVE_ASSET_KEY.to_owned())
            .expect("native asset id should not be empty");

        self.assets
            .get(&asset_id)
            .map(ServiceResponse::from_succeed)
            .unwrap_or_else(|| ServiceError::AssetNotFound(asset_id).into())
    }

    #[cycles(100_00)]
    #[read]
    fn get_asset(&self, ctx: ServiceContext, payload: GetAssetPayload) -> ServiceResponse<Asset> {
        self.assets
            .get(&payload.id)
            .map(ServiceResponse::from_succeed)
            .unwrap_or_else(|| ServiceError::AssetNotFound(payload.id).into())
    }

    #[cycles(100_00)]
    #[read]
    fn get_balance(
        &self,
        ctx: ServiceContext,
        payload: GetBalancePayload,
    ) -> ServiceResponse<GetBalanceResponse> {
        if !self.assets.contains(&payload.asset_id) {
            return ServiceError::AssetNotFound(payload.asset_id).into();
        }

        let asset_balance = self.asset_balance(&payload.user, &payload.asset_id);
        let balance_resp = GetBalanceResponse {
            asset_id: payload.asset_id,
            user:     payload.user,
            balance:  asset_balance.value,
        };

        ServiceResponse::from_succeed(balance_resp)
    }

    #[cycles(100_00)]
    #[read]
    fn get_allowance(
        &self,
        ctx: ServiceContext,
        payload: GetAllowancePayload,
    ) -> ServiceResponse<GetAllowanceResponse> {
        if !self.assets.contains(&payload.asset_id) {
            return ServiceError::AssetNotFound(payload.asset_id).into();
        }

        let asset_balance = self.asset_balance(&payload.grantor, &payload.asset_id);
        let allowance_value = *asset_balance.allowance.get(&payload.grantee).unwrap_or(&0);
        let resp = GetAllowanceResponse {
            asset_id: payload.asset_id,
            grantor:  payload.grantor,
            grantee:  payload.grantee,
            value:    allowance_value,
        };

        ServiceResponse::from_succeed(resp)
    }

    #[cycles(210_00)]
    #[write]
    fn create_asset(
        &mut self,
        ctx: ServiceContext,
        payload: CreateAssetPayload,
    ) -> ServiceResponse<Asset> {
        let caller = ctx.get_caller();
        let payload_json = match serde_json::to_string(&payload) {
            Ok(j) => j,
            Err(err) => return ServiceError::JsonParse(err).into(),
        };

        let asset_id = Hash::digest(Bytes::from(payload_json + &caller.as_hex()));
        if self.assets.contains(&asset_id) {
            return ServiceError::Exists(asset_id).into();
        }

        let asset = Asset {
            id:        asset_id.clone(),
            name:      payload.name,
            symbol:    payload.symbol,
            supply:    payload.supply,
            precision: payload.precision,
            issuer:    caller,
        };
        self.assets.insert(asset_id, asset.clone());
        self.set_account_value(
            &asset.issuer,
            asset.id.clone(),
            AssetBalance::new(payload.supply),
        );

        Self::emit_event(&ctx, &asset);
        ServiceResponse::from_succeed(asset)
    }

    #[cycles(210_00)]
    #[write]
    fn transfer(&mut self, ctx: ServiceContext, payload: TransferPayload) -> ServiceResponse<()> {
        let sender = match Self::extra_caller(&ctx) {
            Ok(s) => s,
            Err(err) => return err.into(),
        };

        let asset_id = payload.asset_id;
        if !self.assets.contains(&asset_id) {
            return ServiceError::AssetNotFound(asset_id).into();
        }

        if let Err(err) = self._transfer(&sender, &payload.to, asset_id.clone(), payload.value) {
            return err.into();
        }

        let event = TransferEvent {
            asset_id,
            from: sender,
            to: payload.to,
            value: payload.value,
        };
        Self::emit_event(&ctx, event)
    }

    #[cycles(210_00)]
    #[write]
    fn approve(&mut self, ctx: ServiceContext, payload: ApprovePayload) -> ServiceResponse<()> {
        let caller = ctx.get_caller();
        if caller == payload.to {
            return ServiceError::ApproveToSelf.into();
        }

        let asset_id = &payload.asset_id;
        if !self.assets.contains(asset_id) {
            return ServiceError::AssetNotFound(payload.asset_id).into();
        }

        let mut caller_asset_balance = self.asset_balance(&caller, &asset_id);
        caller_asset_balance
            .allowance
            .entry(payload.to.clone())
            .and_modify(|e| *e = payload.value)
            .or_insert(payload.value);
        self.set_account_value(&caller, asset_id.to_owned(), caller_asset_balance);

        let event = ApproveEvent {
            asset_id: payload.asset_id,
            grantor:  caller,
            grantee:  payload.to,
            value:    payload.value,
        };
        Self::emit_event(&ctx, event)
    }

    #[cycles(210_00)]
    #[write]
    fn transfer_from(
        &mut self,
        ctx: ServiceContext,
        payload: TransferFromPayload,
    ) -> ServiceResponse<()> {
        let caller = match Self::extra_caller(&ctx) {
            Ok(s) => s,
            Err(err) => return err.into(),
        };

        let asset_id = &payload.asset_id;
        if !self.assets.contains(&asset_id) {
            return ServiceError::AssetNotFound(payload.asset_id).into();
        }

        let mut asset_balance = self.asset_balance(&payload.sender, &asset_id);
        let allowance_balance = *asset_balance.allowance.entry(caller.clone()).or_insert(0);
        if allowance_balance < payload.value {
            return ServiceError::LackOfBalance {
                expect: payload.value,
                real:   allowance_balance,
            }
            .into();
        }

        let after_allowance_balance = allowance_balance - payload.value;
        asset_balance
            .allowance
            .entry(caller.clone())
            .and_modify(|e| *e = after_allowance_balance)
            .or_insert(after_allowance_balance);
        self.set_account_value(&payload.sender, asset_id.to_owned(), asset_balance);

        if let Err(err) = self._transfer(
            &payload.sender,
            &payload.recipient,
            asset_id.to_owned(),
            payload.value,
        ) {
            return err.into();
        }

        let event = TransferFromEvent {
            asset_id: payload.asset_id,
            caller,
            sender: payload.sender,
            recipient: payload.recipient,
            value: payload.value,
        };
        Self::emit_event(&ctx, event)
    }

    fn _transfer(
        &mut self,
        sender: &Address,
        recipient: &Address,
        asset_id: Hash,
        value: u64,
    ) -> Result<(), ServiceError> {
        if sender == recipient {
            return Err(ServiceError::TransferToSelf);
        }

        let mut sender_asset_balance = self.asset_balance(sender, &asset_id);
        let sender_balance = sender_asset_balance.value;
        if sender_balance < value {
            return Err(ServiceError::LackOfBalance {
                expect: value,
                real:   sender_balance,
            });
        }

        let mut to_asset_balance: AssetBalance = self
            .get_account_value(recipient, &asset_id)
            .unwrap_or_default();

        let (v, overflow) = to_asset_balance.value.overflowing_add(value);
        if overflow {
            return Err(ServiceError::BalanceOverflow);
        }
        to_asset_balance.value = v;

        self.set_account_value(recipient, asset_id.clone(), to_asset_balance);

        let (v, overflow) = sender_balance.overflowing_sub(value);
        if overflow {
            return Err(ServiceError::BalanceOverflow);
        }

        sender_asset_balance.value = v;
        self.set_account_value(sender, asset_id, sender_asset_balance);

        Ok(())
    }

    fn asset_balance(&self, account: &Address, asset_id: &Hash) -> AssetBalance {
        self.get_account_value(account, asset_id)
            .unwrap_or_default()
    }

    fn extra_caller(ctx: &ServiceContext) -> Result<Address, ServiceError> {
        match ctx.get_extra() {
            Some(extra) => {
                let opt_str = String::from_utf8(extra.to_vec()).ok();
                let opt_addr = opt_str.map(|str| Address::from_hex(&str).ok());

                match opt_addr.flatten() {
                    Some(addr) => Ok(addr),
                    None => Err(ServiceError::NotHexCaller),
                }
            }
            None => Ok(ctx.get_caller()),
        }
    }

    fn emit_event<T: Serialize>(ctx: &ServiceContext, event: T) -> ServiceResponse<()> {
        match serde_json::to_string(&event) {
            Err(err) => ServiceError::JsonParse(err).into(),
            Ok(json) => {
                ctx.emit_event(json);
                ServiceResponse::from_succeed(())
            }
        }
    }
}

#[derive(Debug, Display)]
pub enum ServiceError {
    #[display(fmt = "Not found asset, id {:?}", _0)]
    AssetNotFound(Hash),

    #[display(fmt = "Lack of balance, expect {:?} real {:?}", expect, real)]
    LackOfBalance { expect: u64, real: u64 },

    #[display(fmt = "Parsing payload to json failed {:?}", _0)]
    JsonParse(serde_json::Error),

    #[display(fmt = "Asset {:?} already exists", _0)]
    Exists(Hash),

    #[display(fmt = "Fee not enough")]
    FeeNotEnough,

    #[display(fmt = "Balance overflow")]
    BalanceOverflow,

    #[display(fmt = "Transfer to self")]
    TransferToSelf,

    #[display(fmt = "Approve to self")]
    ApproveToSelf,

    #[display(fmt = "Sender address is not hex")]
    NotHexCaller,
}

impl ServiceError {
    fn code(&self) -> u64 {
        match self {
            ServiceError::AssetNotFound(_) => 101,
            ServiceError::LackOfBalance { .. } => 102,
            ServiceError::JsonParse(_) => 103,
            ServiceError::Exists(_) => 104,
            ServiceError::FeeNotEnough => 105,
            ServiceError::BalanceOverflow => 106,
            ServiceError::TransferToSelf => 107,
            ServiceError::ApproveToSelf => 108,
            ServiceError::NotHexCaller => 109,
        }
    }
}

impl<T: Default> From<ServiceError> for ServiceResponse<T> {
    fn from(err: ServiceError) -> ServiceResponse<T> {
        ServiceResponse::from_error(err.code(), err.to_string())
    }
}
