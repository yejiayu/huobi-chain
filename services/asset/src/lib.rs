#[cfg(test)]
mod tests;
pub mod types;

use std::ops::{Deref, DerefMut};

use bytes::Bytes;
use derive_more::Display;
use serde::Serialize;

use binding_macro::{cycles, genesis, service, write};
use protocol::traits::{ExecutorParams, ServiceResponse, ServiceSDK, StoreMap, StoreUint64};
use protocol::types::{Address, Hash, ServiceContext};

use crate::types::{
    ApproveEvent, ApprovePayload, Asset, AssetBalance, BurnAsset, BurnEvent, CreateAssetPayload,
    GetAllowancePayload, GetAllowanceResponse, GetAssetPayload, GetBalancePayload,
    GetBalanceResponse, InitGenesisPayload, MintAsset, MintEvent, NewAdmin, TransferEvent,
    TransferFromEvent, TransferFromPayload, TransferPayload,
};

const ADMIN_KEY: &str = "asset_service_admin";
const NATIVE_ASSET_KEY: &str = "native_asset";

macro_rules! require_admin {
    ($sdk:expr, $ctx:expr) => {{
        let admin: Address = $sdk
            .get_value(&ADMIN_KEY.to_owned())
            .expect("admin not found");

        if admin != $ctx.get_caller() {
            return ServiceError::Unauthorized.into();
        }
    }};
}

macro_rules! require_asset_exists {
    ($service:expr, $asset_id:expr) => {
        if !$service.assets.contains(&$asset_id) {
            return ServiceError::AssetNotFound($asset_id).into();
        }
    };
}

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
        if let Err(e) = payload.verify() {
            panic!(e);
        }

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
        self.set_value(ADMIN_KEY.to_owned(), payload.admin);

        self.fee.set(payload.fee);
        self.set_account_value(&asset.issuer, asset.id, AssetBalance::new(payload.supply))
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
        require_asset_exists!(self, payload.asset_id);

        let user_balance = self.asset_balance(&payload.user, &payload.asset_id);

        ServiceResponse::from_succeed(GetBalanceResponse {
            asset_id: payload.asset_id,
            user:     payload.user,
            balance:  *user_balance,
        })
    }

    #[cycles(100_00)]
    #[read]
    fn get_allowance(
        &self,
        ctx: ServiceContext,
        payload: GetAllowancePayload,
    ) -> ServiceResponse<GetAllowanceResponse> {
        require_asset_exists!(self, payload.asset_id);

        let grantor_balance = self.asset_balance(&payload.grantor, &payload.asset_id);
        let grantee_allowance = grantor_balance.allowance(&payload.grantee);

        ServiceResponse::from_succeed(GetAllowanceResponse {
            asset_id: payload.asset_id,
            grantor:  payload.grantor,
            grantee:  payload.grantee,
            value:    grantee_allowance,
        })
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

        let balance = AssetBalance::new(payload.supply);
        self.set_account_value(&asset.issuer, asset.id.clone(), balance);

        Self::emit_event(&ctx, &asset);
        ServiceResponse::from_succeed(asset)
    }

    #[cycles(210_00)]
    #[write]
    fn transfer(&mut self, ctx: ServiceContext, payload: TransferPayload) -> ServiceResponse<()> {
        require_asset_exists!(self, payload.asset_id);

        let sender = match Self::extra_caller(&ctx) {
            Ok(s) => s,
            Err(err) => return err.into(),
        };

        let asset_id = payload.asset_id;
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
        require_asset_exists!(self, payload.asset_id);

        let caller = ctx.get_caller();
        if caller == payload.to {
            return ServiceError::ApproveToSelf.into();
        }

        let asset_id = &payload.asset_id;
        let mut caller_balance = self.asset_balance(&caller, &asset_id);

        caller_balance.update_allowance(payload.to.clone(), payload.value);
        self.set_account_value(&caller, asset_id.to_owned(), caller_balance);

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
        require_asset_exists!(self, payload.asset_id);

        let caller = match Self::extra_caller(&ctx) {
            Ok(s) => s,
            Err(err) => return err.into(),
        };

        let asset_id = &payload.asset_id;
        let mut sender_balance = self.asset_balance(&payload.sender, &asset_id);

        let caller_allowance = sender_balance.allowance(&caller);
        if caller_allowance < payload.value {
            return ServiceError::LackOfBalance {
                expect: payload.value,
                real:   caller_allowance,
            }
            .into();
        }

        let (checked_allowance, overflow) = caller_allowance.overflowing_sub(payload.value);
        if overflow {
            return ServiceError::BalanceOverflow.into();
        }

        sender_balance.update_allowance(caller.clone(), checked_allowance);
        self.set_account_value(&payload.sender, asset_id.to_owned(), sender_balance);

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

    #[cycles(210_00)]
    #[write]
    fn change_admin(&mut self, ctx: ServiceContext, payload: NewAdmin) -> ServiceResponse<()> {
        require_admin!(self.sdk, &ctx);

        self.sdk
            .set_value(ADMIN_KEY.to_owned(), payload.addr.clone());

        Self::emit_event(&ctx, payload)
    }

    // TODO: verify proof
    #[cycles(210_00)]
    #[write]
    fn mint(&mut self, ctx: ServiceContext, payload: MintAsset) -> ServiceResponse<()> {
        require_admin!(self.sdk, &ctx);
        require_asset_exists!(self, payload.asset_id);

        let mut recipient_balance = self.asset_balance(&payload.to, &payload.asset_id);

        if let Err(e) = recipient_balance.checked_add(payload.amount) {
            return e.into();
        }

        self.set_account_value(&payload.to, payload.asset_id.clone(), recipient_balance);

        Self::emit_event(&ctx, MintEvent {
            asset_id: payload.asset_id,
            to:       payload.to,
            amount:   payload.amount,
        })
    }

    // TODO: verify proof
    #[cycles(210_00)]
    #[write]
    fn burn(&mut self, ctx: ServiceContext, payload: BurnAsset) -> ServiceResponse<()> {
        require_asset_exists!(self, payload.asset_id);

        let mut burner_balance = self.asset_balance(&ctx.get_caller(), &payload.asset_id);
        if let Err(e) = burner_balance.checked_sub(payload.amount) {
            return e.into();
        }

        self.set_account_value(&ctx.get_caller(), payload.asset_id.clone(), burner_balance);

        Self::emit_event(&ctx, BurnEvent {
            asset_id: payload.asset_id,
            from:     ctx.get_caller(),
            amount:   payload.amount,
        })
    }

    fn _transfer(
        &mut self,
        sender: &Address,
        recipient: &Address,
        asset_id: Hash,
        value: u64,
    ) -> Result<(), ServiceError> {
        if sender == recipient {
            return Ok(());
        }

        let mut sender_balance = self.asset_balance(sender, &asset_id);
        if sender_balance < value {
            return Err(ServiceError::LackOfBalance {
                expect: value,
                real:   sender_balance.value,
            });
        }

        sender_balance.checked_sub(value)?;
        self.set_account_value(sender, asset_id.clone(), sender_balance);

        let mut recipient_balance = self.asset_balance(recipient, &asset_id);
        recipient_balance.checked_add(value)?;
        self.set_account_value(recipient, asset_id, recipient_balance);

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

    #[cfg(test)]
    fn admin(&self) -> Address {
        self.sdk
            .get_value(&ADMIN_KEY.to_owned())
            .expect("admin not found")
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

    #[display(fmt = "Approve to self")]
    ApproveToSelf,

    #[display(fmt = "Sender address is not hex")]
    NotHexCaller,

    #[display(fmt = "Unauthorized")]
    Unauthorized,
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
            ServiceError::ApproveToSelf => 107,
            ServiceError::NotHexCaller => 108,
            ServiceError::Unauthorized => 109,
        }
    }
}

impl<T: Default> From<ServiceError> for ServiceResponse<T> {
    fn from(err: ServiceError) -> ServiceResponse<T> {
        ServiceResponse::from_error(err.code(), err.to_string())
    }
}
