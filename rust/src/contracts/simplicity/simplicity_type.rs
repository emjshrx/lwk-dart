use lwk_simplicity::simplicityhl;
use simplicityhl::parse::ParseFromStr;
use simplicityhl::types::TypeConstructible;

use crate::api::error::LwkError;

/// Simplicity type descriptor.
#[derive(Clone, Debug)]
pub struct SimplicityType {
    pub(crate) inner: simplicityhl::ResolvedType,
}

impl SimplicityType {
    pub fn u1() -> SimplicityType {
        SimplicityType {
            inner: simplicityhl::ResolvedType::u1(),
        }
    }

    pub fn u8() -> SimplicityType {
        SimplicityType {
            inner: simplicityhl::ResolvedType::u8(),
        }
    }

    pub fn u16() -> SimplicityType {
        SimplicityType {
            inner: simplicityhl::ResolvedType::u16(),
        }
    }

    pub fn u32() -> SimplicityType {
        SimplicityType {
            inner: simplicityhl::ResolvedType::u32(),
        }
    }

    pub fn u64() -> SimplicityType {
        SimplicityType {
            inner: simplicityhl::ResolvedType::u64(),
        }
    }

    pub fn u128() -> SimplicityType {
        SimplicityType {
            inner: simplicityhl::ResolvedType::u128(),
        }
    }

    pub fn u256() -> SimplicityType {
        SimplicityType {
            inner: simplicityhl::ResolvedType::u256(),
        }
    }

    pub fn boolean() -> SimplicityType {
        SimplicityType {
            inner: simplicityhl::ResolvedType::boolean(),
        }
    }

    pub fn either(left: &SimplicityType, right: &SimplicityType) -> SimplicityType {
        SimplicityType {
            inner: simplicityhl::ResolvedType::either(left.inner.clone(), right.inner.clone()),
        }
    }

    pub fn option(inner: &SimplicityType) -> SimplicityType {
        SimplicityType {
            inner: simplicityhl::ResolvedType::option(inner.inner.clone()),
        }
    }

    pub fn tuple(elements: Vec<SimplicityType>) -> SimplicityType {
        SimplicityType {
            inner: simplicityhl::ResolvedType::tuple(elements.iter().map(|e| e.inner.clone())),
        }
    }

    pub fn from_string(type_str: String) -> anyhow::Result<SimplicityType, LwkError> {
        let inner = simplicityhl::ResolvedType::parse_from_str(&type_str)
            .map_err(|e| LwkError { msg: e.to_string() })?;
        Ok(SimplicityType { inner })
    }

    pub fn to_string_repr(&self) -> String {
        self.inner.to_string()
    }

    pub(crate) fn inner(&self) -> &simplicityhl::ResolvedType {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simplicity_type_string_roundtrip() {
        let left = SimplicityType::u32();
        let right = SimplicityType::u64();
        let tuple = SimplicityType::tuple(vec![left, right]);
        let parsed = SimplicityType::from_string(tuple.to_string_repr()).unwrap();
        assert_eq!(parsed.to_string_repr(), tuple.to_string_repr());
    }
}
