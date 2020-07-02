#[cfg(test)]
mod tests;
mod types;

use std::cell::RefCell;
use std::rc::Rc;

use bytes::Bytes;
use derive_more::{Display, From};
use serde::Serialize;

use binding_macro::{cycles, genesis, hook_after, service, tx_hook_after};
use protocol::traits::{ExecutorParams, ServiceResponse, ServiceSDK, StoreMap};
use protocol::types::{Address, Metadata, ServiceContext, ServiceContextParams};

use crate::types::{
    AccmulateProfitPayload, Asset, DiscountLevel, GovernanceInfo, InitGenesisPayload,
    MinerChargeConfig, RecordProfitEvent, SetAdminEvent, SetAdminPayload, SetGovernInfoEvent,
    SetGovernInfoPayload, SetMinerEvent, TransferFromPayload, UpdateIntervalEvent,
    UpdateIntervalPayload, UpdateMetadataEvent, UpdateMetadataPayload, UpdateRatioEvent,
    UpdateRatioPayload, UpdateValidatorsEvent, UpdateValidatorsPayload,
};
use std::convert::{From, TryInto};

#[cfg(not(test))]
use crate::types::{GetBalancePayload, GetBalanceResponse};

const INFO_KEY: &str = "admin";
const TX_FEE_INLET_KEY: &str = "fee_address";
const MINER_PROFIT_OUTLET_KEY: &str = "miner_address";
const MILLION: u64 = 1_000_000;
const HUNDRED: u64 = 100;
static ADMISSION_TOKEN: Bytes = Bytes::from_static(b"governance");

pub struct GovernanceService<SDK> {
    sdk:     SDK,
    profits: Box<dyn StoreMap<Address, u64>>,
    miners:  Box<dyn StoreMap<Address, Address>>,
}

#[service]
impl<SDK: ServiceSDK> GovernanceService<SDK> {
    pub fn new(mut sdk: SDK) -> Self {
        let profits: Box<dyn StoreMap<Address, u64>> = sdk.alloc_or_recover_map("profit");
        let miners: Box<dyn StoreMap<Address, Address>> = sdk.alloc_or_recover_map("miner_address");
        Self {
            sdk,
            profits,
            miners,
        }
    }

    #[genesis]
    fn init_genesis(&mut self, payload: InitGenesisPayload) {
        assert!(self.profits.is_empty());

        let mut info = payload.info;
        info.tx_fee_discount.sort();
        self.sdk.set_value(INFO_KEY.to_string(), info);
        self.sdk
            .set_value(TX_FEE_INLET_KEY.to_string(), payload.tx_fee_inlet_address);
        self.sdk.set_value(
            MINER_PROFIT_OUTLET_KEY.to_string(),
            payload.miner_profit_outlet_address,
        );

        for miner in payload.miner_charge_map.into_iter() {
            self.miners
                .insert(miner.address, miner.miner_charge_address);
        }
    }

    #[cycles(210_00)]
    #[read]
    fn get_admin_address(&self, ctx: ServiceContext) -> ServiceResponse<Address> {
        let info: GovernanceInfo = self
            .sdk
            .get_value(&INFO_KEY.to_owned())
            .expect("Admin should not be none");

        ServiceResponse::from_succeed(info.admin)
    }

    #[cycles(210_00)]
    #[read]
    fn get_govern_info(&self, ctx: ServiceContext) -> ServiceResponse<GovernanceInfo> {
        let info: GovernanceInfo = self
            .sdk
            .get_value(&INFO_KEY.to_owned())
            .expect("Admin should not be none");

        ServiceResponse::from_succeed(info)
    }

    #[cycles(210_00)]
    #[read]
    fn get_tx_failure_fee(&self, ctx: ServiceContext) -> ServiceResponse<u64> {
        let info: GovernanceInfo = self
            .sdk
            .get_value(&INFO_KEY.to_owned())
            .expect("Admin should not be none");

        ServiceResponse::from_succeed(info.tx_failure_fee)
    }

    #[cycles(210_00)]
    #[read]
    fn get_tx_floor_fee(&self, ctx: ServiceContext) -> ServiceResponse<u64> {
        let info: GovernanceInfo = self
            .sdk
            .get_value(&INFO_KEY.to_owned())
            .expect("Admin should not be none");

        ServiceResponse::from_succeed(info.tx_floor_fee)
    }

    #[cycles(210_00)]
    #[write]
    fn set_admin(&mut self, ctx: ServiceContext, payload: SetAdminPayload) -> ServiceResponse<()> {
        if !self.is_admin(&ctx) {
            return ServiceError::NonAuthorized.into();
        }

        let mut info: GovernanceInfo = self
            .sdk
            .get_value(&INFO_KEY.to_owned())
            .expect("Admin should not be none");
        info.admin = payload.admin.clone();

        self.sdk.set_value(INFO_KEY.to_owned(), info);

        let event = SetAdminEvent {
            topic: "Set New Admin".to_owned(),
            admin: payload.admin,
        };
        Self::emit_event(&ctx, event)
    }

    #[cycles(210_00)]
    #[write]
    fn set_govern_info(
        &mut self,
        ctx: ServiceContext,
        payload: SetGovernInfoPayload,
    ) -> ServiceResponse<()> {
        if !self.is_admin(&ctx) {
            return ServiceError::NonAuthorized.into();
        }

        let mut info = payload.inner;
        info.tx_fee_discount.sort();
        self.sdk.set_value(INFO_KEY.to_owned(), info.clone());

        let event = SetGovernInfoEvent {
            topic: "Set New Govern Info".to_owned(),
            info,
        };
        Self::emit_event(&ctx, event)
    }

    #[cycles(210_00)]
    #[write]
    fn set_miner(
        &mut self,
        ctx: ServiceContext,
        payload: MinerChargeConfig,
    ) -> ServiceResponse<()> {
        if ctx.get_caller() != payload.address {
            return ServiceResponse::from_error(103, "Can only set own miner address".to_owned());
        }

        self.miners.insert(
            payload.address.clone(),
            payload.miner_charge_address.clone(),
        );
        let event = SetMinerEvent {
            topic: "Set New Miner Info".to_owned(),
            info:  payload,
        };
        Self::emit_event(&ctx, event)
    }

    #[cycles(210_00)]
    #[write]
    fn update_metadata(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateMetadataPayload,
    ) -> ServiceResponse<()> {
        if !self.is_admin(&ctx) {
            return ServiceError::NonAuthorized.into();
        }

        if let Err(err) = self.write_metadata(&ctx, payload.clone()) {
            return err;
        }

        Self::emit_event(&ctx, UpdateMetadataEvent::from(payload))
    }

    #[cycles(210_00)]
    #[write]
    fn update_validators(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateValidatorsPayload,
    ) -> ServiceResponse<()> {
        if !self.is_admin(&ctx) {
            return ServiceError::NonAuthorized.into();
        }

        let mut metadata = match self.get_metadata(&ctx) {
            Ok(m) => m,
            Err(resp) => return resp,
        };

        metadata.verifier_list = payload.verifier_list.clone();
        if let Err(err) = self.write_metadata(&ctx, UpdateMetadataPayload::from(metadata)) {
            return err;
        }

        Self::emit_event(&ctx, UpdateValidatorsEvent::from(payload))
    }

    #[cycles(210_00)]
    #[write]
    fn update_interval(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateIntervalPayload,
    ) -> ServiceResponse<()> {
        if !self.is_admin(&ctx) {
            return ServiceError::NonAuthorized.into();
        }

        let mut metadata = match self.get_metadata(&ctx) {
            Ok(m) => m,
            Err(resp) => return resp,
        };

        metadata.interval = payload.interval;
        if let Err(err) = self.write_metadata(&ctx, UpdateMetadataPayload::from(metadata)) {
            return err;
        }

        Self::emit_event(&ctx, UpdateIntervalEvent::from(payload))
    }

    #[cycles(210_00)]
    #[write]
    fn update_ratio(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateRatioPayload,
    ) -> ServiceResponse<()> {
        if !self.is_admin(&ctx) {
            return ServiceError::NonAuthorized.into();
        }

        let mut metadata = match self.get_metadata(&ctx) {
            Ok(m) => m,
            Err(resp) => return resp,
        };
        metadata.propose_ratio = payload.propose_ratio;
        metadata.prevote_ratio = payload.prevote_ratio;
        metadata.precommit_ratio = payload.precommit_ratio;
        metadata.brake_ratio = payload.brake_ratio;

        if let Err(err) = self.write_metadata(&ctx, UpdateMetadataPayload::from(metadata)) {
            return err;
        }

        Self::emit_event(&ctx, UpdateRatioEvent::from(payload))
    }

    #[cycles(210_00)]
    #[write]
    fn accumulate_profit(
        &mut self,
        ctx: ServiceContext,
        payload: AccmulateProfitPayload,
    ) -> ServiceResponse<()> {
        let address = payload.address;
        let new_profit = payload.accumulated_profit;

        if let Some(profit) = self.profits.get(&address) {
            if let Some(profit_sum) = profit.checked_add(new_profit) {
                self.profits.insert(address.clone(), profit_sum);
            } else {
                return ServiceResponse::from_error(101, "profit overflow".to_owned());
            }
        } else {
            self.profits.insert(address.clone(), new_profit);
        }

        Self::emit_event(&ctx, RecordProfitEvent {
            owner:  address,
            amount: new_profit,
        });

        ServiceResponse::from_succeed(())
    }

    fn calc_profit_records(&mut self, _ctx: &ServiceContext) -> u64 {
        let profits = self
            .profits
            .iter()
            .map(|i| (i.0.clone(), i.1))
            .collect::<Vec<_>>();

        let mut profit_sum = 0u64;

        for (owner, profit) in profits.iter() {
            profit_sum = profit_sum.checked_add(profit.to_owned()).unwrap();
            self.profits.insert(owner.clone(), 0);
        }

        profit_sum
    }

    #[cfg(test)]
    fn profits_len(&self) -> u64 {
        self.profits.len()
    }

    #[tx_hook_after]
    fn handle_tx_fee(&mut self, ctx: ServiceContext) {
        let asset = self
            .get_native_asset(&ctx)
            .expect("Can not get native asset");

        let tx_fee = self.calc_tx_fee(&ctx);

        // Reset accumulated profit
        let keys = self.profits.iter().map(|(k, _)| k).collect::<Vec<_>>();
        for key in keys {
            self.profits.remove(&key);
        }

        let tx_fee_inlet_address: Address =
            self.sdk.get_value(&TX_FEE_INLET_KEY.to_owned()).unwrap();

        let _ = self.transfer_from(&ctx, TransferFromPayload {
            asset_id:  asset.id,
            sender:    ctx.get_caller(),
            recipient: tx_fee_inlet_address,
            value:     tx_fee,
        });
    }

    #[hook_after]
    fn handle_miner_profit(&mut self, params: &ExecutorParams) {
        let info: GovernanceInfo = self
            .sdk
            .get_value(&INFO_KEY.to_owned())
            .expect("Admin should not be none");
        let sender_address: Address = self
            .sdk
            .get_value(&MINER_PROFIT_OUTLET_KEY.to_owned())
            .expect("send miner fee address should not be none");

        let ctx_params = ServiceContextParams {
            tx_hash:         None,
            nonce:           None,
            cycles_limit:    params.cycles_limit,
            cycles_price:    1,
            cycles_used:     Rc::new(RefCell::new(0)),
            caller:          sender_address.clone(),
            height:          params.height,
            service_name:    String::new(),
            service_method:  String::new(),
            service_payload: String::new(),
            extra:           None,
            timestamp:       params.timestamp,
            events:          Rc::new(RefCell::new(vec![])),
        };

        let ctx = ServiceContext::new(ctx_params);
        let asset = self
            .get_native_asset(&ctx)
            .expect("Can not get native asset");

        let recipient_addr = if let Some(addr) = self.miners.get(&params.proposer) {
            addr
        } else {
            params.proposer.clone()
        };

        let payload = TransferFromPayload {
            asset_id:  asset.id,
            sender:    sender_address,
            recipient: recipient_addr,
            value:     info.miner_benefit,
        };

        let _ = self.transfer_from(&ctx, payload);
    }

    fn calc_tx_fee(&mut self, ctx: &ServiceContext) -> u64 {
        let profit = self.calc_profit_records(ctx);

        let info: GovernanceInfo = self
            .sdk
            .get_value(&INFO_KEY.to_owned())
            .expect("Admin should not be none");

        let fee: u64 = (profit as u128 * info.profit_deduct_rate_per_million as u128
            / MILLION as u128)
            .try_into()
            .unwrap_or_else(|err| panic!(err));

        let fee = self.calc_discount_fee(ctx, fee, &info.tx_fee_discount);

        fee.max(info.tx_floor_fee)
    }

    fn calc_discount_fee(
        &self,
        ctx: &ServiceContext,
        origin_fee: u64,
        discount_level: &[DiscountLevel],
    ) -> u64 {
        let mut discount = HUNDRED;

        let balance = self.get_balance(ctx);

        for level in discount_level.iter().rev() {
            if balance >= level.threshold {
                discount = level.discount_percent;
                break;
            }
        }

        let fee: u64 = (origin_fee as u128 * discount as u128 / HUNDRED as u128)
            .try_into()
            .unwrap_or_else(|err| panic!(err));
        fee
    }

    #[cfg(not(test))]
    fn get_balance(&self, ctx: &ServiceContext) -> u64 {
        let asset = self
            .get_native_asset(ctx)
            .expect("Can not get native asset");

        let payload = GetBalancePayload {
            asset_id: asset.id,
            user:     ctx.get_caller(),
        };
        let payload = serde_json::to_string(&payload)
            .expect("Can not marshall GetBalancePayload in calc_discount_fee");

        let resp = self.sdk.read(
            &ctx,
            Some(ADMISSION_TOKEN.clone()),
            "asset",
            "get_balance",
            payload.as_str(),
        );

        if resp.is_error() {
            panic!("query balance fails")
        }

        let balance = serde_json::from_str::<GetBalanceResponse>(resp.succeed_data.as_str())
            .expect("Can not unmarshall GetBalancePayload in calc_discount_fee");
        balance.balance
    }

    fn is_admin(&self, ctx: &ServiceContext) -> bool {
        let caller = ctx.get_caller();
        let info: GovernanceInfo = self
            .sdk
            .get_value(&INFO_KEY.to_string())
            .expect("Admin should not be none");

        info.admin == caller
    }

    fn get_metadata(&self, ctx: &ServiceContext) -> Result<Metadata, ServiceResponse<()>> {
        let resp = self.sdk.read(ctx, None, "metadata", "get_metadata", "");
        if resp.is_error() {
            return Err(ServiceResponse::from_error(resp.code, resp.error_message));
        }

        let meta_json: String = resp.succeed_data;
        let meta = serde_json::from_str(&meta_json).map_err(ServiceError::JsonParse)?;
        Ok(meta)
    }

    fn write_metadata(
        &mut self,
        ctx: &ServiceContext,
        payload: UpdateMetadataPayload,
    ) -> Result<(), ServiceResponse<()>> {
        let payload_json = match serde_json::to_string(&payload) {
            Ok(j) => j,
            Err(err) => return Err(ServiceError::JsonParse(err).into()),
        };

        let resp = self.sdk.write(
            &ctx,
            Some(ADMISSION_TOKEN.clone()),
            "metadata",
            "update_metadata",
            &payload_json,
        );

        if resp.is_error() {
            Err(ServiceResponse::from_error(resp.code, resp.error_message))
        } else {
            Ok(())
        }
    }

    fn transfer_from(
        &mut self,
        ctx: &ServiceContext,
        payload: TransferFromPayload,
    ) -> Result<(), ServiceResponse<()>> {
        let payload_json = match serde_json::to_string(&payload) {
            Ok(j) => j,
            Err(err) => return Err(ServiceError::JsonParse(err).into()),
        };

        let resp = self.sdk.write(
            &ctx,
            Some(ADMISSION_TOKEN.clone()),
            "asset",
            "transfer_from",
            &payload_json,
        );

        if resp.is_error() {
            Err(ServiceResponse::from_error(resp.code, resp.error_message))
        } else {
            Ok(())
        }
    }

    fn get_native_asset(&self, ctx: &ServiceContext) -> Result<Asset, ServiceResponse<Asset>> {
        let resp = self.sdk.read(
            &ctx,
            Some(ADMISSION_TOKEN.clone()),
            "asset",
            "get_native_asset",
            "",
        );

        if resp.is_error() {
            Err(ServiceResponse::from_error(resp.code, resp.error_message))
        } else {
            let ret: Asset = serde_json::from_str(&resp.succeed_data).unwrap();
            Ok(ret)
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

    #[cfg(test)]
    fn get_balance(&self, _ctx: &ServiceContext) -> u64 {
        100_000
    }
}

#[derive(Debug, Display, From)]
pub enum ServiceError {
    NonAuthorized,

    #[display(fmt = "Parsing payload to json failed {:?}", _0)]
    JsonParse(serde_json::Error),
}

impl ServiceError {
    fn code(&self) -> u64 {
        match self {
            ServiceError::NonAuthorized => 101,
            ServiceError::JsonParse(_) => 102,
        }
    }
}

impl<T: Default> From<ServiceError> for ServiceResponse<T> {
    fn from(err: ServiceError) -> ServiceResponse<T> {
        ServiceResponse::from_error(err.code(), err.to_string())
    }
}
