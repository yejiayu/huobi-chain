use crate::ServiceError;

use derive_more::{Display, From};
use muta_codec_derive::RlpFixedCodec;
use protocol::{
    fixed_codec::{FixedCodec, FixedCodecError},
    types::Address,
    Bytes, ProtocolResult,
};
use serde::de::{Deserializer, Error, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};

use std::{
    collections::HashMap,
    fmt,
    ops::{Deref, DerefMut},
    str::FromStr,
};

#[derive(Debug, From, Display)]
#[display(fmt = "{}", _0)]
pub struct BadPayload(&'static str);

impl From<BadPayload> for ServiceError {
    fn from(err: BadPayload) -> ServiceError {
        ServiceError::BadPayload(err.0.to_owned())
    }
}

pub trait Validate {
    fn validate(&self) -> Result<(), ServiceError>;
}

pub trait Validated {}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Display)]
#[display(fmt = "{}", _0)]
pub struct TagString(String);

// No way to create invalid TagString from public fn
impl Validated for TagString {}

impl TagString {
    pub fn validate(s: &str) -> Result<(), &'static str> {
        if s.chars().count() > 12 {
            return Err("tag length exceed 12");
        }

        // 'NULL' is reversed keyword, make sure that a tag array doesn't
        // contain it.
        if s.chars().count() == 4 && s.to_uppercase() == "NULL" {
            return Err("tag null is not allowed");
        }

        for (i, c) in s.chars().enumerate() {
            if i == 0 && !c.is_ascii_alphanumeric() {
                return Err("tag must start with alpha latter");
            }

            if !c.is_ascii_alphanumeric() && c != '_' {
                return Err("tag only support ascii alpha, number, underscore");
            }
        }

        Ok(())
    }
}

impl FromStr for TagString {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s)?;

        Ok(TagString(s.to_owned()))
    }
}

impl rlp::Encodable for TagString {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        self.0.rlp_append(s)
    }
}

impl rlp::Decodable for TagString {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let name = String::decode(rlp)?;
        TagString::validate(&name).map_err(rlp::DecoderError::Custom)?;

        Ok(TagString(name))
    }
}

impl FixedCodec for TagString {
    fn encode_fixed(&self) -> ProtocolResult<Bytes> {
        Ok(rlp::encode(self).into())
    }

    fn decode_fixed(bytes: Bytes) -> ProtocolResult<Self> {
        Ok(rlp::decode(&bytes).map_err(FixedCodecError::from)?)
    }
}

impl<'de> Deserialize<'de> for TagString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringVisitor;

        impl<'de> Visitor<'de> for StringVisitor {
            type Value = TagString;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("tag string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(StringVisitor)
    }
}

impl Deref for TagString {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TagString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Into<String> for TagString {
    fn into(self) -> String {
        self.0
    }
}

pub type TagName = TagString;

#[derive(Debug, Serialize, PartialEq, Eq, Clone)]
#[serde(deny_unknown_fields)]
pub struct NoneEmptyVec<T: Validated>(Vec<T>);

impl<T: Validated> Into<Vec<T>> for NoneEmptyVec<T> {
    fn into(self) -> Vec<T> {
        self.0
    }
}

impl<T: Validated> IntoIterator for NoneEmptyVec<T> {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Validated> Deref for NoneEmptyVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Validated> DerefMut for NoneEmptyVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl NoneEmptyVec<TagString> {
    pub fn validate(tags: &[TagString]) -> Result<(), &'static str> {
        if tags.is_empty() {
            return Err("tag array is empty");
        }

        Ok(())
    }
}

impl FixedCodec for NoneEmptyVec<TagString> {
    fn encode_fixed(&self) -> ProtocolResult<Bytes> {
        Ok(rlp::encode_list(self).into())
    }

    fn decode_fixed(bytes: Bytes) -> ProtocolResult<Self> {
        let rlp = rlp::Rlp::new(&bytes);
        let tags = rlp.as_list().map_err(FixedCodecError::from)?;

        NoneEmptyVec::validate(&tags)
            .map_err(rlp::DecoderError::Custom)
            .map_err(FixedCodecError::from)?;

        Ok(NoneEmptyVec(tags))
    }
}

impl<'de> Deserialize<'de> for NoneEmptyVec<TagString> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TagVisitor;

        impl<'de> Visitor<'de> for TagVisitor {
            type Value = NoneEmptyVec<TagString>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("tag array")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = seq.next_element::<Option<TagString>>()? {
                    vec.extend(elem);
                }

                NoneEmptyVec::validate(&vec).map_err(A::Error::custom)?;
                Ok(NoneEmptyVec(vec))
            }
        }

        deserializer.deserialize_seq(TagVisitor)
    }
}

#[derive(Debug, Serialize, PartialEq, Eq, Hash, Clone, Display)]
#[display(fmt = "{}", _0)]
pub struct OrgName(String);

impl OrgName {
    pub fn validate(s: &str) -> Result<(), &'static str> {
        if s.chars().count() > 12 {
            return Err("org name exceed 12 chars");
        }

        for (i, c) in s.chars().enumerate() {
            if i == 0 && !c.is_ascii_alphanumeric() {
                return Err("org name must start with alpha latter");
            }

            if !c.is_ascii_alphanumeric() && c != '_' {
                return Err("org name only support ascii alpha, number, underscore");
            }
        }

        Ok(())
    }
}

impl FromStr for OrgName {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s)?;

        Ok(OrgName(s.to_owned()))
    }
}

impl rlp::Encodable for OrgName {
    fn rlp_append(&self, s: &mut rlp::RlpStream) {
        self.0.rlp_append(s)
    }
}

impl rlp::Decodable for OrgName {
    fn decode(rlp: &rlp::Rlp) -> Result<Self, rlp::DecoderError> {
        let name = String::decode(rlp)?;
        OrgName::validate(&name).map_err(rlp::DecoderError::Custom)?;

        Ok(OrgName(name))
    }
}

impl FixedCodec for OrgName {
    fn encode_fixed(&self) -> ProtocolResult<Bytes> {
        Ok(rlp::encode(self).into())
    }

    fn decode_fixed(bytes: Bytes) -> ProtocolResult<Self> {
        Ok(rlp::decode(&bytes).map_err(FixedCodecError::from)?)
    }
}

impl Deref for OrgName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OrgName {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'de> Deserialize<'de> for OrgName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StringVisitor;

        impl<'de> Visitor<'de> for StringVisitor {
            type Value = OrgName;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("tag string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(E::custom)
            }
        }

        deserializer.deserialize_str(StringVisitor)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Genesis {
    pub org_name:        OrgName,
    pub org_description: String,
    pub org_admin:       Address,
    pub supported_tags:  Vec<TagName>,
    pub service_admin:   Address,
}

impl Validate for Genesis {
    fn validate(&self) -> Result<(), ServiceError> {
        if self.org_description.len() >= 256 {
            return Err(BadPayload("description length exceed 256").into());
        }

        if self.org_admin == Address::default() {
            return Err(BadPayload("invalid org admin address").into());
        }

        if self.service_admin == Address::default() {
            return Err(BadPayload("invalid service admin address").into());
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, RlpFixedCodec)]
pub struct KycOrgInfo {
    pub name:           OrgName,
    pub description:    String,
    pub admin:          Address,
    pub supported_tags: Vec<TagName>,
    pub approved:       bool,
}

impl Validate for KycOrgInfo {
    // Note: TagName and OrgName is already validated during deserialization,
    // and there's not way to create invalid TagName from public function.
    fn validate(&self) -> Result<(), ServiceError> {
        if self.description.len() >= 256 {
            return Err(BadPayload("description length exceed 256").into());
        }

        if self.admin == Address::default() {
            return Err(BadPayload("invalid org admin address").into());
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct FixedTagList(NoneEmptyVec<TagString>);

impl FixedTagList {
    pub fn from_vec(tags: Vec<TagString>) -> Result<Self, &'static str> {
        Self::validate(&tags)?;

        Ok(FixedTagList(NoneEmptyVec(tags)))
    }

    pub fn validate(tags: &[TagString]) -> Result<(), &'static str> {
        NoneEmptyVec::validate(tags)?;

        if tags.len() > 6 {
            return Err("tag array length exceed 6");
        }

        Ok(())
    }
}

impl FixedCodec for FixedTagList {
    fn encode_fixed(&self) -> ProtocolResult<Bytes> {
        self.0.encode_fixed()
    }

    fn decode_fixed(bytes: Bytes) -> ProtocolResult<Self> {
        let tags = NoneEmptyVec::decode_fixed(bytes)?;

        FixedTagList::validate(&tags)
            .map_err(rlp::DecoderError::Custom)
            .map_err(FixedCodecError::from)?;

        Ok(FixedTagList(tags))
    }
}

impl<'de> Deserialize<'de> for FixedTagList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TagVisitor;

        impl<'de> Visitor<'de> for TagVisitor {
            type Value = FixedTagList;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("tag array")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = seq.next_element::<Option<TagString>>()? {
                    vec.extend(elem);
                }

                let tags = FixedTagList::from_vec(vec).map_err(A::Error::custom)?;
                Ok(tags)
            }
        }

        deserializer.deserialize_seq(TagVisitor)
    }
}

impl Into<Vec<String>> for FixedTagList {
    fn into(self) -> Vec<String> {
        self.0.into_iter().map(Into::into).collect()
    }
}

impl Deref for FixedTagList {
    type Target = NoneEmptyVec<TagString>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FixedTagList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for FixedTagList {
    type IntoIter = std::vec::IntoIter<Self::Item>;
    type Item = TagString;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct KycUserInfo {
    pub tags: HashMap<TagName, FixedTagList>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeOrgApproved {
    pub org_name: OrgName,
    pub approved: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RegisterNewOrg {
    pub name:           OrgName,
    pub description:    String,
    pub admin:          Address,
    pub supported_tags: Vec<TagName>,
}

impl Validate for RegisterNewOrg {
    fn validate(&self) -> Result<(), ServiceError> {
        if self.description.len() >= 256 {
            return Err(BadPayload("description length exceed 256").into());
        }

        if self.admin == Address::default() {
            return Err(BadPayload("invalid admin address").into());
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NewOrgEvent {
    pub name:           OrgName,
    pub supported_tags: Vec<TagString>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UpdateOrgSupportTags {
    pub org_name:       OrgName,
    pub supported_tags: Vec<TagName>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UpdateUserTags {
    pub org_name: OrgName,
    pub user:     Address,
    pub tags:     HashMap<TagName, FixedTagList>,
}

impl Validate for UpdateUserTags {
    fn validate(&self) -> Result<(), ServiceError> {
        if self.user == Address::default() {
            return Err(BadPayload("invalid user address").into());
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserTags {
    pub org_name: OrgName,
    pub user:     Address,
}

impl Validate for GetUserTags {
    fn validate(&self) -> Result<(), ServiceError> {
        if self.user == Address::default() {
            Err(BadPayload("invalid user address").into())
        } else {
            Ok(())
        }
    }
}
#[derive(Debug, Deserialize)]
pub struct EvalUserTagExpression {
    pub user:       Address,
    pub expression: String,
}

impl Validate for EvalUserTagExpression {
    fn validate(&self) -> Result<(), ServiceError> {
        if self.user == Address::default() {
            Err(BadPayload("invalid user address").into())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeOrgAdmin {
    pub name:      OrgName,
    pub new_admin: Address,
}

impl Validate for ChangeOrgAdmin {
    fn validate(&self) -> Result<(), ServiceError> {
        if self.new_admin == Address::default() {
            Err(BadPayload("invalid admin address").into())
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeServiceAdmin {
    pub new_admin: Address,
}

impl Validate for ChangeServiceAdmin {
    fn validate(&self) -> Result<(), ServiceError> {
        if self.new_admin == Address::default() {
            Err(BadPayload("invalid admin address").into())
        } else {
            Ok(())
        }
    }
}
