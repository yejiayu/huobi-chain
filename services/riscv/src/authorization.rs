use std::{cell::RefCell, rc::Rc};

use protocol::traits::{ServiceSDK, StoreBool, StoreMap};
use protocol::types::Address;

use crate::types::{Authorizer, InitGenesisPayload};

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Deploy,
    Contract,
}

pub struct Authorization {
    enabled:       Box<dyn StoreBool>,
    deploy_auth:   Box<dyn StoreMap<Address, Authorizer>>,
    contract_auth: Box<dyn StoreMap<Address, Authorizer>>,
}

impl Authorization {
    pub fn new<SDK: ServiceSDK + 'static>(sdk: &Rc<RefCell<SDK>>) -> Self {
        let enabled = sdk
            .borrow_mut()
            .alloc_or_recover_bool("enable_authorization");
        let deploy_auth = sdk.borrow_mut().alloc_or_recover_map("deploy_auth");
        let contract_auth = sdk.borrow_mut().alloc_or_recover_map("contract_auth");

        Authorization {
            enabled,
            deploy_auth,
            contract_auth,
        }
    }

    // # Panic
    pub fn init_genesis(&mut self, payload: InitGenesisPayload) {
        self.enabled.set(payload.enable_authorization);

        for addr in payload.deploy_auth {
            self.deploy_auth.insert(addr, Authorizer::none());
        }
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
