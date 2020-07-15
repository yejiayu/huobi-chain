mod error;
mod expression;
#[cfg(test)]
mod tests;
mod types;
use error::ServiceError;

use expression::traits::ExpressionDataFeed;
use types::{
    ChangeOrgAdmin, ChangeOrgApproved, ChangeServiceAdmin, EvalUserTagExpression, FixedTagList,
    Genesis, GetUserTags, KycOrgInfo, NewOrgEvent, OrgName, RegisterNewOrg, TagName,
    UpdateOrgSupportTags, UpdateUserTags, Validate,
};

use binding_macro::{cycles, genesis, service};
use derive_more::Constructor;
use muta_codec_derive::RlpFixedCodec;
use protocol::{
    fixed_codec::{FixedCodec, FixedCodecError},
    traits::{ExecutorParams, ServiceResponse, ServiceSDK, StoreMap},
    types::{Address, ServiceContext},
    Bytes, ProtocolResult,
};
use serde::Serialize;

use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, DerefMut},
};

const KYC_SERVICE_ADMIN_KEY: &str = "kyc_service_admin";

macro_rules! require_service_admin {
    ($service:expr, $ctx:expr) => {{
        let admin = if let Some(tmp) = $service
            .sdk
            .get_value::<_, Address>(&KYC_SERVICE_ADMIN_KEY.to_owned())
        {
            tmp
        } else {
            return ServiceError::NonAuthorized.into();
        };

        if admin != $ctx.get_caller() {
            return ServiceError::NonAuthorized.into();
        }
    }};
}

macro_rules! require_org_exists {
    ($service:expr, $org_name:expr) => {
        if !$service.orgs.contains(&$org_name) {
            return ServiceError::OrgNotFound($org_name).into();
        }
    };
}

#[macro_export]
macro_rules! sub_cycles {
    ($ctx:expr, $cycles:expr) => {
        if !$ctx.sub_cycles($cycles) {
            return ServiceError::OutOfCycles.into();
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq, RlpFixedCodec, Constructor)]
struct UserTagNamesKey {
    org_name: OrgName,
    user:     Address,
}

#[derive(Debug, Clone, PartialEq, Eq, RlpFixedCodec, Constructor)]
struct UserTagsKey {
    org_name: OrgName,
    user:     Address,
    tag_name: TagName,
}

#[derive(Debug, PartialEq, Eq, RlpFixedCodec)]
struct TagNameList(Vec<TagName>);

impl IntoIterator for TagNameList {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = TagName;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

// Required for RlpFixedCodec derive
impl Deref for TagNameList {
    type Target = Vec<TagName>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TagNameList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct KycService<SDK> {
    sdk:            SDK,
    orgs:           Box<dyn StoreMap<OrgName, KycOrgInfo>>,
    orgs_approved:  Box<dyn StoreMap<OrgName, bool>>,
    user_tag_names: Box<dyn StoreMap<UserTagNamesKey, TagNameList>>,
    user_tags:      Box<dyn StoreMap<UserTagsKey, FixedTagList>>,
}

#[service]
impl<SDK: ServiceSDK> KycService<SDK> {
    pub fn new(mut sdk: SDK) -> Self {
        let orgs = sdk.alloc_or_recover_map("kyc_orgs");
        let orgs_approved = sdk.alloc_or_recover_map("kyc_orgs_approved");
        let user_tag_names = sdk.alloc_or_recover_map("kyc_user");
        let user_tags = sdk.alloc_or_recover_map("kyc_user_tags");

        Self {
            sdk,
            orgs,
            orgs_approved,
            user_tag_names,
            user_tags,
        }
    }

    #[genesis]
    fn init_genesis(&mut self, genesis: Genesis) {
        if let Err(e) = genesis.validate() {
            panic!(e);
        }

        let org = KycOrgInfo {
            name:           genesis.org_name.clone(),
            description:    genesis.org_description,
            admin:          genesis.org_admin,
            supported_tags: genesis.supported_tags,
            approved:       true,
        };
        self.orgs.insert(genesis.org_name.to_owned(), org);
        self.orgs_approved.insert(genesis.org_name.to_owned(), true);

        self.sdk
            .set_value(KYC_SERVICE_ADMIN_KEY.to_owned(), genesis.service_admin);
    }

    #[cycles(21_000)]
    #[read]
    fn get_orgs(&self, ctx: ServiceContext) -> ServiceResponse<Vec<OrgName>> {
        let mut org_names = Vec::new();

        for (org_name, _) in self.orgs_approved.iter() {
            if !ctx.sub_cycles(10_000u64) {
                return ServiceError::OutOfCycles.into();
            }

            org_names.push(org_name);
        }

        ServiceResponse::from_succeed(org_names)
    }

    // Note: Use Option to provide default value require by ServiceResponse
    #[cycles(21_000)]
    #[read]
    fn get_org_info(
        &self,
        ctx: ServiceContext,
        org_name: OrgName,
    ) -> ServiceResponse<Option<KycOrgInfo>> {
        require_org_exists!(self, org_name);

        // Impossible, already ensure org exists
        let mut org = self.orgs.get(&org_name).unwrap();
        org.approved = self.orgs_approved.get(&org_name).unwrap_or_else(|| false);

        ServiceResponse::from_succeed(Some(org))
    }

    #[cycles(21_000)]
    #[read]
    fn get_org_supported_tags(
        &self,
        ctx: ServiceContext,
        org_name: OrgName,
    ) -> ServiceResponse<Vec<TagName>> {
        require_org_exists!(self, org_name);

        // Impossible, already ensure org exists
        let org = self.orgs.get(&org_name).unwrap();

        let required_cycles = org.supported_tags.len() as u64 * 10_000;
        sub_cycles!(ctx, required_cycles);

        ServiceResponse::from_succeed(org.supported_tags)
    }

    #[cycles(21_000)]
    #[read]
    fn get_user_tags(
        &self,
        ctx: ServiceContext,
        payload: GetUserTags,
    ) -> ServiceResponse<HashMap<TagName, FixedTagList>> {
        if let Err(e) = payload.validate() {
            return e.into();
        }

        require_org_exists!(self, payload.org_name);

        let tag_names_key = UserTagNamesKey::new(payload.org_name.clone(), payload.user.clone());
        let tag_names: Vec<TagName> = match self.user_tag_names.get(&tag_names_key) {
            Some(names) => names.0,
            None => return ServiceResponse::from_succeed(HashMap::new()),
        };

        let required_cycles = tag_names.len() * 10_000;
        sub_cycles!(ctx, required_cycles as u64);

        let mut user_tags = HashMap::with_capacity(tag_names.len());
        for tag_name in tag_names.into_iter() {
            let tags_key = UserTagsKey::new(
                payload.org_name.clone(),
                payload.user.clone(),
                tag_name.to_owned(),
            );

            if let Some(tags) = self.user_tags.get(&tags_key) {
                user_tags.insert(tag_name, tags);
            }
        }

        ServiceResponse::from_succeed(user_tags)
    }

    #[cycles(21_000)]
    #[read]
    fn eval_user_tag_expression(
        &self,
        ctx: ServiceContext,
        payload: EvalUserTagExpression,
    ) -> ServiceResponse<bool> {
        if let Err(e) = payload.validate() {
            return e.into();
        }

        let required_cycles = payload.expression.len() * 10_000;
        sub_cycles!(ctx, required_cycles as u64);

        let evaluated = match expression::evaluate(self, payload.user, payload.expression) {
            Ok(r) => r,
            Err(e) => return ServiceError::Expression(e).into(),
        };

        ServiceResponse::from_succeed(evaluated)
    }

    #[cycles(21_000)]
    #[write]
    fn change_org_approved(
        &mut self,
        ctx: ServiceContext,
        payload: ChangeOrgApproved,
    ) -> ServiceResponse<()> {
        require_service_admin!(self, &ctx);
        require_org_exists!(self, payload.org_name);

        self.orgs_approved
            .insert(payload.org_name.clone(), payload.approved);

        Self::emit_event(&ctx, "ChangeOrgApproved".to_owned(), payload)
    }

    #[cycles(21_000)]
    #[write]
    fn change_service_admin(
        &mut self,
        ctx: ServiceContext,
        payload: ChangeServiceAdmin,
    ) -> ServiceResponse<()> {
        if let Err(e) = payload.validate() {
            return e.into();
        }

        require_service_admin!(self, &ctx);

        self.sdk
            .set_value(KYC_SERVICE_ADMIN_KEY.to_owned(), payload.new_admin);
        ServiceResponse::from_succeed(())
    }

    #[cycles(21_000)]
    #[write]
    fn change_org_admin(
        &mut self,
        ctx: ServiceContext,
        payload: ChangeOrgAdmin,
    ) -> ServiceResponse<()> {
        require_org_exists!(self, payload.name);

        let mut org = self.orgs.get(&payload.name).unwrap();
        if ctx.get_caller() != org.admin {
            return ServiceError::NonAuthorized.into();
        }

        org.admin = payload.new_admin.clone();
        self.orgs.insert(payload.name.clone(), org);

        Self::emit_event(&ctx, "ChangeOrgAdmin".to_owned(), payload)
    }

    #[cycles(21_000)]
    #[write]
    fn register_org(
        &mut self,
        ctx: ServiceContext,
        new_org: RegisterNewOrg,
    ) -> ServiceResponse<()> {
        require_service_admin!(self, &ctx);

        if let Err(e) = new_org.validate() {
            return e.into();
        }
        if self.orgs.contains(&new_org.name) {
            return ServiceError::OrgAlreadyExists.into();
        }

        let required_cycles = {
            let string_bytes =
                new_org.name.len() + new_org.description.len() + new_org.admin.as_bytes().len();
            let tags = new_org.supported_tags.len();

            string_bytes * 1000 + tags * 10_000
        };
        sub_cycles!(ctx, required_cycles as u64);

        let org = KycOrgInfo {
            name:           new_org.name.clone(),
            description:    new_org.description,
            admin:          new_org.admin,
            supported_tags: new_org.supported_tags.clone(),
            approved:       false,
        };

        self.orgs.insert(new_org.name.to_owned(), org);
        self.orgs_approved.insert(new_org.name.to_owned(), false);

        Self::emit_event(&ctx, "RegisterOrg".to_owned(), NewOrgEvent {
            name:           new_org.name,
            supported_tags: new_org.supported_tags,
        })
    }

    #[cycles(21_000)]
    #[write]
    fn update_supported_tags(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateOrgSupportTags,
    ) -> ServiceResponse<()> {
        require_service_admin!(self, &ctx);
        require_org_exists!(self, payload.org_name);

        let required_cycles = payload.supported_tags.len() * 10_000;
        sub_cycles!(ctx, required_cycles as u64);

        // Impossible, already checked by require_org_exists!()
        let mut org = self.orgs.get(&payload.org_name).unwrap();
        org.supported_tags = payload.supported_tags.clone();
        self.orgs.insert(payload.org_name.clone(), org);

        Self::emit_event(&ctx, "UpdateSupportedTag".to_owned(), payload)
    }

    #[cycles(21_000)]
    #[write]
    fn update_user_tags(
        &mut self,
        ctx: ServiceContext,
        payload: UpdateUserTags,
    ) -> ServiceResponse<()> {
        require_org_exists!(self, payload.org_name);

        if !self
            .orgs_approved
            .get(&payload.org_name)
            .unwrap_or_else(|| false)
        {
            return ServiceError::UnapprovedOrg.into();
        }

        // Impossible, already checked by require_org_exists!()
        let org = self.orgs.get(&payload.org_name).unwrap();
        if org.admin != ctx.get_caller() {
            return ServiceError::NonAuthorized.into();
        }

        // Update tags
        let tag_names_key = UserTagNamesKey::new(payload.org_name.clone(), payload.user.clone());

        let tag_names = TagNameList(payload.tags.keys().cloned().collect::<Vec<_>>());
        if tag_names.len() > 0 {
            let required_cycles = tag_names.len() * 10_000;
            sub_cycles!(ctx, required_cycles as u64);
        }

        let mut old_tag_names = self
            .user_tag_names
            .get(&tag_names_key)
            .map(|v| v.into_iter().collect::<HashSet<_>>())
            .unwrap_or_else(HashSet::new);

        self.user_tag_names.insert(tag_names_key, tag_names);

        for (tag_name, tags) in payload.tags.iter() {
            old_tag_names.remove(&tag_name);

            let required_cycles = tags.len() * 10_000;
            sub_cycles!(ctx, required_cycles as u64);

            let tags_key = UserTagsKey::new(
                payload.org_name.clone(),
                payload.user.clone(),
                tag_name.to_owned(),
            );
            self.user_tags.insert(tags_key, tags.to_owned());
        }

        // Clear unused tags
        for tag_name in old_tag_names.into_iter() {
            let tags_key =
                UserTagsKey::new(payload.org_name.clone(), payload.user.clone(), tag_name);
            self.user_tags.remove(&tags_key);
        }

        Self::emit_event(&ctx, "UpdateUserTag".to_owned(), payload)
    }

    fn emit_event<T: Serialize>(
        ctx: &ServiceContext,
        name: String,
        event: T,
    ) -> ServiceResponse<()> {
        match serde_json::to_string(&event) {
            Err(err) => ServiceError::Serde(err).into(),
            Ok(json) => {
                ctx.emit_event(name, json);
                ServiceResponse::from_succeed(())
            }
        }
    }
}

impl<SDK: ServiceSDK> ExpressionDataFeed for KycService<SDK> {
    fn get_tags(
        &self,
        user: Address,
        kyc: String,
        tag: String,
    ) -> Result<Vec<String>, &'static str> {
        let org_name = kyc.parse()?;
        let tag_name = tag.parse()?;

        if !self.orgs_approved.get(&org_name).unwrap_or_else(|| false) {
            return Err("unapproved org");
        }

        let user_tags_key = UserTagsKey::new(org_name, user, tag_name);
        let tags = match self.user_tags.get(&user_tags_key) {
            Some(tags) => tags.into_iter().map(Into::into).collect(),
            None => vec!["NULL".to_owned()],
        };

        Ok(tags)
    }
}
