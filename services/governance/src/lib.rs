#[cfg(test)]
mod tests;
mod types;

use std::cell::RefCell;
use std::rc::Rc;

use bytes::Bytes;
use derive_more::{Display, From};
use serde::Serialize;

use asset::types::{Asset, HookTransferFromPayload};
use asset::Assets;
use binding_macro::{
    cycles, genesis, hook_after, hook_before, service, tx_hook_after, tx_hook_before,
};
use metadata::types::UpdateMetadataPayload;
use metadata::MetaData;
use protocol::traits::{ExecutorParams, ServiceResponse, ServiceSDK, StoreMap};
use protocol::types::{Address, Metadata, ServiceContext, ServiceContextParams};

use crate::types::{
    AccumulateProfitPayload, ConsumedTxFee, DiscountLevel, GovernanceInfo, InitGenesisPayload,
    MinerChargeConfig, RecordProfitEvent, SetAdminEvent, SetAdminPayload, SetGovernInfoEvent,
    SetGovernInfoPayload, SetMinerEvent, UpdateIntervalEvent, UpdateIntervalPayload,
    UpdateMetadataEvent, UpdateRatioEvent, UpdateRatioPayload, UpdateValidatorsEvent,
    UpdateValidatorsPayload,
};
use std::convert::{From, TryInto};

const INFO_KEY: &str = "admin";
const BLOCK_MINER_KEY: &str = "block_miner";
const MINER_PROFIT_OUTLET_KEY: &str = "miner_address";
const MILLION: u64 = 1_000_000;
const HUNDRED: u64 = 100;
static ADMISSION_TOKEN: Bytes = Bytes::from_static(b"governance");

macro_rules! require_admin {
    ($service: expr, $ctx:expr) => {
        if !$service.is_admin($ctx) {
            return ServiceError::NonAuthorized.into();
        }
    };
}

macro_rules! get_info {
    ($service:expr) => {{
        let tmp = $service
            .sdk
            .get_value::<_, GovernanceInfo>(&INFO_KEY.to_owned());
        if tmp.is_none() {
            return ServiceError::MissingInfo.into();
        }
        tmp.unwrap()
    }};
}

macro_rules! impl_governance {
    ($self: expr, $method: ident, $ctx: expr) => {{
        let res = $self.$method($ctx.clone());
        if res.is_error() {
            Err(ServiceResponse::from_error(res.code, res.error_message))
        } else {
            Ok(res.succeed_data)
        }
    }};
    ($self: expr, $method: ident, $ctx: expr, $payload: expr) => {{
        let res = $self.$method($ctx.clone(), $payload);
        if res.is_error() {
            Err(ServiceResponse::from_error(res.code, res.error_message))
        } else {
            Ok(res.succeed_data)
        }
    }};
}

pub trait Governance {
    fn get_info(&self, ctx: &ServiceContext) -> Result<GovernanceInfo, ServiceResponse<()>>;

    fn declare_profit(
        &mut self,
        ctx: &ServiceContext,
        payload: AccumulateProfitPayload,
    ) -> Result<(), ServiceResponse<()>>;
}

pub struct GovernanceService<A, M, SDK> {
    sdk:     SDK,
    profits: Box<dyn StoreMap<Address, u64>>,
    miners:  Box<dyn StoreMap<Address, Address>>,

    asset:    A,
    metadata: M,
}

impl<A, M, SDK> Governance for GovernanceService<A, M, SDK>
where
    A: Assets,
    M: MetaData,
    SDK: ServiceSDK,
{
    fn get_info(&self, ctx: &ServiceContext) -> Result<GovernanceInfo, ServiceResponse<()>> {
        impl_governance!(self, get_govern_info, ctx)
    }

    fn declare_profit(
        &mut self,
        ctx: &ServiceContext,
        payload: AccumulateProfitPayload,
    ) -> Result<(), ServiceResponse<()>> {
        impl_governance!(self, accumulate_profit, ctx, payload)
    }
}

#[service]
impl<A, M, SDK> GovernanceService<A, M, SDK>
where
    A: Assets,
    M: MetaData,
    SDK: ServiceSDK,
{
    pub fn new(mut sdk: SDK, asset: A, metadata: M) -> Self {
        let profits: Box<dyn StoreMap<Address, u64>> = sdk.alloc_or_recover_map("profit");
        let miners: Box<dyn StoreMap<Address, Address>> = sdk.alloc_or_recover_map("miner_address");
        Self {
            sdk,
            profits,
            miners,
            asset,
            metadata,
        }
    }

    #[genesis]
    fn init_genesis(&mut self, payload: InitGenesisPayload) {
        assert!(self.profits.is_empty());

        let mut info = payload.info;
        info.tx_fee_discount.sort();
        self.sdk.set_value(INFO_KEY.to_string(), info);
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
    fn get_admin(&self, ctx: ServiceContext) -> ServiceResponse<Address> {
        if let Some(info) = self
            .sdk
            .get_value::<_, GovernanceInfo>(&INFO_KEY.to_owned())
        {
            ServiceResponse::from_succeed(info.admin)
        } else {
            ServiceError::MissingInfo.into()
        }
    }

    #[cycles(210_00)]
    #[read]
    fn get_govern_info(&self, ctx: ServiceContext) -> ServiceResponse<GovernanceInfo> {
        let info = get_info!(self);
        ServiceResponse::from_succeed(info)
    }

    #[cycles(210_00)]
    #[read]
    fn get_tx_failure_fee(&self, ctx: ServiceContext) -> ServiceResponse<u64> {
        let info = get_info!(self);
        ServiceResponse::from_succeed(info.tx_failure_fee)
    }

    #[cycles(210_00)]
    #[read]
    fn get_tx_floor_fee(&self, ctx: ServiceContext) -> ServiceResponse<u64> {
        let info = get_info!(self);
        ServiceResponse::from_succeed(info.tx_floor_fee)
    }

    #[cycles(210_00)]
    #[write]
    fn set_admin(&mut self, ctx: ServiceContext, payload: SetAdminPayload) -> ServiceResponse<()> {
        require_admin!(self, &ctx);
        let mut info = get_info!(self);

        info.admin = payload.admin.clone();
        self.sdk.set_value(INFO_KEY.to_owned(), info);

        let event = SetAdminEvent {
            admin: payload.admin,
        };
        Self::emit_event(&ctx, "SetAdmin".to_owned(), event)
    }

    #[cycles(210_00)]
    #[write]
    fn set_govern_info(
        &mut self,
        ctx: ServiceContext,
        payload: SetGovernInfoPayload,
    ) -> ServiceResponse<()> {
        require_admin!(self, &ctx);

        let mut info = payload.inner;
        info.tx_fee_discount.sort();
        self.sdk.set_value(INFO_KEY.to_owned(), info.clone());

        let event = SetGovernInfoEvent { info };
        Self::emit_event(&ctx, "SetGovernInfo".to_owned(), event)
    }

    #[cycles(210_00)]
    #[write]
    fn set_miner(
        &mut self,
        ctx: ServiceContext,
        payload: MinerChargeConfig,
    ) -> ServiceResponse<()> {
        require_admin!(self, &ctx);

        self.miners.insert(
            payload.address.clone(),
            payload.miner_charge_address.clone(),
        );
        let event = SetMinerEvent { info: payload };
        Self::emit_event(&ctx, "SetMiner".to_owned(), event)
    }

    #[cycles(210_00)]
    #[write]
    fn update_metadata(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateMetadataPayload,
    ) -> ServiceResponse<()> {
        require_admin!(self, &ctx);

        if let Err(err) = self.write_metadata(&ctx, payload.clone()) {
            return err;
        }

        Self::emit_event(
            &ctx,
            "UpdateMetadata".to_owned(),
            UpdateMetadataEvent::from(payload),
        )
    }

    #[cycles(210_00)]
    #[write]
    fn update_validators(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateValidatorsPayload,
    ) -> ServiceResponse<()> {
        require_admin!(self, &ctx);

        let mut metadata = match self.get_metadata(&ctx) {
            Ok(m) => m,
            Err(resp) => return resp,
        };

        metadata.verifier_list = payload.verifier_list.clone();
        if let Err(err) = self.write_metadata(&ctx, UpdateMetadataPayload::from(metadata)) {
            return err;
        }

        Self::emit_event(
            &ctx,
            "UpdateValidators".to_owned(),
            UpdateValidatorsEvent::from(payload),
        )
    }

    #[cycles(210_00)]
    #[write]
    fn update_interval(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateIntervalPayload,
    ) -> ServiceResponse<()> {
        require_admin!(self, &ctx);

        let mut metadata = match self.get_metadata(&ctx) {
            Ok(m) => m,
            Err(resp) => return resp,
        };

        metadata.interval = payload.interval;
        if let Err(err) = self.write_metadata(&ctx, UpdateMetadataPayload::from(metadata)) {
            return err;
        }

        Self::emit_event(
            &ctx,
            "UpdateInterval".to_owned(),
            UpdateIntervalEvent::from(payload),
        )
    }

    #[cycles(210_00)]
    #[write]
    fn update_ratio(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateRatioPayload,
    ) -> ServiceResponse<()> {
        require_admin!(self, &ctx);

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

        Self::emit_event(
            &ctx,
            "UpdateRatio".to_owned(),
            UpdateRatioEvent::from(payload),
        )
    }

    #[cycles(210_00)]
    #[write]
    fn accumulate_profit(
        &mut self,
        ctx: ServiceContext,
        payload: AccumulateProfitPayload,
    ) -> ServiceResponse<()> {
        let address = payload.address;
        let new_profit = payload.accumulated_profit;

        if let Some(profit) = self.profits.get(&address) {
            if let Some(profit_sum) = profit.checked_add(new_profit) {
                self.profits.insert(address.clone(), profit_sum);
            } else {
                return ServiceError::Overflow.into();
            }
        } else {
            self.profits.insert(address.clone(), new_profit);
        }

        Self::emit_event(&ctx, "RecordProfit".to_owned(), RecordProfitEvent {
            owner:  address,
            amount: new_profit,
        });

        ServiceResponse::from_succeed(())
    }

    fn calc_profit_records(&mut self, _ctx: &ServiceContext) -> Result<u64, ServiceError> {
        let profits = self
            .profits
            .iter()
            .map(|i| (i.0.clone(), i.1))
            .collect::<Vec<_>>();

        let mut profit_sum = 0u64;
        for (owner, profit) in profits.iter() {
            if let Some(tmp) = profit_sum.checked_add(profit.to_owned()) {
                profit_sum = tmp;
                self.profits.insert(owner.clone(), 0);
            } else {
                return Err(ServiceError::Overflow);
            }
        }

        Ok(profit_sum)
    }

    #[hook_before]
    fn set_block_miner(&mut self, params: &ExecutorParams) {
        let miner_addr = {
            let opt_miner = self.miners.get(&params.proposer);
            opt_miner.unwrap_or_else(|| params.proposer.clone())
        };

        self.sdk.set_value(BLOCK_MINER_KEY.to_owned(), miner_addr);
    }

    #[tx_hook_before]
    fn pledge_fee(&mut self, ctx: ServiceContext) -> ServiceResponse<String> {
        let info = self
            .sdk
            .get_value::<_, GovernanceInfo>(&INFO_KEY.to_owned());
        let miner_addr = self.sdk.get_value(&BLOCK_MINER_KEY.to_owned());

        if info.is_none() || miner_addr.is_none() {
            return ServiceError::MissingInfo.into();
        }

        let info = info.unwrap();
        let miner_addr = miner_addr.unwrap();
        // clean fee
        let profits = self.profits.iter().map(|pair| pair.0).collect::<Vec<_>>();

        profits
            .into_iter()
            .for_each(|addr| self.profits.insert(addr, 0));

        // Pledge the tx failure fee before executed the transaction.
        let ret = self.hook_transfer_from(&ctx, HookTransferFromPayload {
            sender:    ctx.get_caller(),
            recipient: miner_addr,
            value:     info.tx_failure_fee,
            memo:      "pledge tx failure fee".to_string(),
        });

        if let Err(e) = ret {
            if e.is_error() {
                return ServiceResponse::from_error(e.code, e.error_message);
            }
        }

        ServiceResponse::from_succeed("".to_owned())
    }

    #[tx_hook_after]
    fn deduct_fee(&mut self, ctx: ServiceContext) -> ServiceResponse<String> {
        let tx_fee = self.calc_tx_fee(&ctx);
        if tx_fee.is_err() {
            return tx_fee.err().unwrap().into();
        }

        let miner_addr: Address = match self.sdk.get_value(&BLOCK_MINER_KEY.to_owned()) {
            None => return ServiceError::MissingInfo.into(),
            Some(addr) => addr,
        };

        let info: GovernanceInfo = match self.sdk.get_value(&INFO_KEY.to_owned()) {
            None => return ServiceError::MissingInfo.into(),
            Some(info) => info,
        };

        let tx_fee = tx_fee.unwrap();
        if tx_fee == 0 {
            let resp = Self::emit_event(&ctx, "ConsumedTxFee".to_owned(), ConsumedTxFee {
                caller: ctx.get_caller(),
                miner:  miner_addr,
                amount: info.tx_failure_fee,
            });

            let resp = if resp.is_error() {
                ServiceResponse::from_error(resp.code, resp.error_message)
            } else {
                ServiceResponse::from_succeed("".to_owned())
            };

            return resp;
        }

        let (tx, rx) = if tx_fee > 0 {
            (ctx.get_caller(), miner_addr.clone())
        } else {
            (miner_addr.clone(), ctx.get_caller())
        };

        let ret = self.hook_transfer_from(&ctx, HookTransferFromPayload {
            sender:    tx,
            recipient: rx,
            value:     tx_fee.abs() as u64,
            memo:      "collect tx fee".to_string(),
        });

        if let Err(e) = ret {
            if e.is_error() {
                return ServiceResponse::from_error(e.code, e.error_message);
            }
        }

        let amount = match consumed_tx_fee(info.tx_failure_fee, tx_fee) {
            Err(err) => {
                return err.into();
            }
            Ok(v) => v,
        };

        let resp = Self::emit_event(&ctx, "ConsumedTxFee".to_owned(), ConsumedTxFee {
            caller: ctx.get_caller(),
            miner: miner_addr,
            amount,
        });

        if resp.is_error() {
            ServiceResponse::from_error(resp.code, resp.error_message)
        } else {
            ServiceResponse::from_succeed("".to_owned())
        }
    }

    #[hook_after]
    fn handle_miner_profit(&mut self, params: &ExecutorParams) {
        let info = self
            .sdk
            .get_value::<_, GovernanceInfo>(&INFO_KEY.to_owned());

        let sender_address = self
            .sdk
            .get_value::<_, Address>(&MINER_PROFIT_OUTLET_KEY.to_owned());

        if info.is_none() || sender_address.is_none() {
            return;
        }

        let info = info.unwrap();
        let sender_address = sender_address.unwrap();

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
            extra:           Some(ADMISSION_TOKEN.clone()),
            timestamp:       params.timestamp,
            events:          Rc::new(RefCell::new(vec![])),
        };

        let recipient_addr = if let Some(addr) = self.miners.get(&params.proposer) {
            addr
        } else {
            params.proposer.clone()
        };

        let payload = HookTransferFromPayload {
            sender:    sender_address,
            recipient: recipient_addr,
            value:     info.miner_benefit,
            memo:      "pay miner fee".to_string(),
        };

        let _ = self.hook_transfer_from(&ServiceContext::new(ctx_params), payload);
    }

    fn calc_tx_fee(&mut self, ctx: &ServiceContext) -> Result<i128, ServiceError> {
        if ctx.canceled() {
            return Ok(0i128);
        }

        let info: GovernanceInfo = self
            .sdk
            .get_value(&INFO_KEY.to_owned())
            .ok_or_else(|| ServiceError::NonAuthorized)?;

        let profit = self.calc_profit_records(ctx)?;
        let fee: u64 = (profit as u128 * info.profit_deduct_rate_per_million as u128
            / MILLION as u128)
            .try_into()
            .map_err(|_| ServiceError::Overflow)?;

        let fee = self
            .calc_discount_fee(ctx, fee, &info.tx_fee_discount)?
            .max(info.tx_floor_fee) as i128;

        Ok(fee - info.tx_failure_fee as i128)
    }

    fn calc_discount_fee(
        &self,
        ctx: &ServiceContext,
        origin_fee: u64,
        discount_level: &[DiscountLevel],
    ) -> Result<u64, ServiceError> {
        let mut discount = HUNDRED;
        let balance = self.get_balance(ctx)?;

        for level in discount_level.iter().rev() {
            if balance >= level.threshold {
                discount = level.discount_percent;
                break;
            }
        }

        let fee: u64 = (origin_fee as u128 * discount as u128 / HUNDRED as u128)
            .try_into()
            .map_err(|_| ServiceError::Overflow)?;
        Ok(fee)
    }

    // #[cfg(test)]
    // fn block_miner(&self) -> Address {
    //     self.sdk
    //         .get_value::<_, Address>(&BLOCK_MINER_KEY.to_owned())
    //         .unwrap()
    // }

    // #[cfg(test)]
    // fn profits_len(&self) -> u64 {
    //     self.profits.len()
    // }

    // #[cfg(test)]
    // fn get_balance(&self, _ctx: &ServiceContext) -> Result<u64, ServiceError> {
    //     Ok(100_000)
    // }

    fn get_balance(&self, ctx: &ServiceContext) -> Result<u64, ServiceError> {
        use asset::types::GetBalancePayload;

        let asset = self
            .get_native_asset(ctx)
            .map_err(|_| ServiceError::QueryBalance)?;

        let balance = self
            .asset
            .balance(ctx, GetBalancePayload {
                asset_id: asset.id,
                user:     ctx.get_caller(),
            })
            .map_err(|_| ServiceError::QueryBalance)?;

        Ok(balance.balance)
    }

    fn is_admin(&self, ctx: &ServiceContext) -> bool {
        self.sdk
            .get_value::<_, GovernanceInfo>(&INFO_KEY.to_string())
            .map_or(false, |info| info.admin == ctx.get_caller())
    }

    fn get_metadata(&self, ctx: &ServiceContext) -> Result<Metadata, ServiceResponse<()>> {
        self.metadata.get(ctx)
    }

    fn write_metadata(
        &mut self,
        ctx: &ServiceContext,
        payload: UpdateMetadataPayload,
    ) -> Result<(), ServiceResponse<()>> {
        self.metadata.update(ctx, payload)
    }

    fn hook_transfer_from(
        &mut self,
        ctx: &ServiceContext,
        payload: HookTransferFromPayload,
    ) -> Result<(), ServiceResponse<()>> {
        self.asset.hook_transfer_from(ctx, payload)
    }

    fn get_native_asset(&self, ctx: &ServiceContext) -> Result<Asset, ServiceResponse<()>> {
        self.asset.native_asset(ctx)
    }

    fn emit_event<T: Serialize>(
        ctx: &ServiceContext,
        name: String,
        event: T,
    ) -> ServiceResponse<()> {
        match serde_json::to_string(&event) {
            Err(err) => ServiceError::JsonParse(err).into(),
            Ok(json) => {
                ctx.emit_event(name, json);
                ServiceResponse::from_succeed(())
            }
        }
    }
}

fn consumed_tx_fee(tx_failure_fee: u64, tx_fee: i128) -> Result<u64, ServiceError> {
    if tx_fee > 0 {
        let (checked_sum, overflow) = tx_failure_fee.overflowing_add(tx_fee.abs() as u64);
        if overflow {
            return Err(ServiceError::Overflow);
        }

        Ok(checked_sum)
    } else {
        let (checked_val, overflow) = tx_failure_fee.overflowing_sub(tx_fee.abs() as u64);
        if overflow {
            return Err(ServiceError::Overflow);
        }

        Ok(checked_val)
    }
}

#[derive(Debug, Display, From)]
pub enum ServiceError {
    NonAuthorized,

    #[display(fmt = "Can not get governance info")]
    MissingInfo,

    #[display(fmt = "calc overflow")]
    Overflow,

    #[display(fmt = "query balance failed")]
    QueryBalance,

    #[display(fmt = "Parsing payload to json failed {:?}", _0)]
    JsonParse(serde_json::Error),
}

impl ServiceError {
    fn code(&self) -> u64 {
        match self {
            ServiceError::NonAuthorized => 101,
            ServiceError::JsonParse(_) => 102,
            ServiceError::MissingInfo => 103,
            ServiceError::Overflow => 104,
            ServiceError::QueryBalance => 105,
        }
    }
}

impl<T: Default> From<ServiceError> for ServiceResponse<T> {
    fn from(err: ServiceError) -> ServiceResponse<T> {
        ServiceResponse::from_error(err.code(), err.to_string())
    }
}
