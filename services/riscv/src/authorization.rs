use crate::types::{Authorizer, InitGenesisPayload};

use protocol::{
    traits::{ServiceSDK, StoreBool, StoreMap},
    types::{Address, ServiceContext},
};

use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Deploy,
    Contract,
}

pub struct Authorization {
    enabled:       Box<dyn StoreBool>,
    admins:        Box<dyn StoreMap<Address, bool>>,
    deploy_auth:   Box<dyn StoreMap<Address, Authorizer>>,
    contract_auth: Box<dyn StoreMap<Address, Authorizer>>,
}

impl Authorization {
    pub fn new<SDK: ServiceSDK + 'static>(sdk: &Rc<RefCell<SDK>>) -> Self {
        let enabled = sdk
            .borrow_mut()
            .alloc_or_recover_bool("enable_authorization");
        let admins = sdk.borrow_mut().alloc_or_recover_map("admins");
        let deploy_auth = sdk.borrow_mut().alloc_or_recover_map("deploy_auth");
        let contract_auth = sdk.borrow_mut().alloc_or_recover_map("contract_auth");

        Authorization {
            enabled,
            admins,
            deploy_auth,
            contract_auth,
        }
    }

    // # Panic
    pub fn init_genesis(&mut self, payload: InitGenesisPayload) {
        if payload.enable_authorization && payload.admins.is_empty() {
            panic!(
                "If riscv service authorization is enabled, you should set at least one admin in genesis.toml"
            );
        }

        self.enabled.set(payload.enable_authorization);

        for addr in payload.deploy_auth {
            self.deploy_auth.insert(addr, Authorizer::none());
        }
        for addr in payload.admins {
            self.admins.insert(addr, true);
        }
    }

    pub fn is_admin(&self, ctx: &ServiceContext) -> bool {
        self.admins.contains(&ctx.get_caller())
    }

    pub fn granted(&self, address: &Address, kind: Kind) -> bool {
        if !self.enabled.get() {
            return true;
        }

        self.kind(kind).contains(address)
    }

    pub fn authorizer(&self, address: &Address, kind: Kind) -> Authorizer {
        self.kind(kind)
            .get(&address)
            .unwrap_or_else(Authorizer::none)
    }

    pub fn contains(&self, address: &Address, kind: Kind) -> bool {
        self.kind(kind).contains(&address)
    }

    pub fn revoke(&mut self, address: &Address, kind: Kind) {
        self.kind_mut(kind).remove(address);
    }

    pub fn grant(&mut self, address: Address, kind: Kind, authorizer: Authorizer) {
        self.kind_mut(kind).insert(address, authorizer);
    }

    fn kind(&self, kind: Kind) -> &dyn StoreMap<Address, Authorizer> {
        match kind {
            Kind::Deploy => self.deploy_auth.as_ref(),
            Kind::Contract => self.contract_auth.as_ref(),
        }
    }

    fn kind_mut(&mut self, kind: Kind) -> &mut dyn StoreMap<Address, Authorizer> {
        match kind {
            Kind::Deploy => self.deploy_auth.as_mut(),
            Kind::Contract => self.contract_auth.as_mut(),
        }
    }
}
