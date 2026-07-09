use lwk_simplicity::simplicityhl;
use lwk_simplicity::simplicityhl::num::U256;
use lwk_simplicity::simplicityhl::value::ValueConstructible;

use crate::api::error::LwkError;
use crate::contracts::simplicity::SimplicityType;

/// Typed Simplicity value.
#[derive(Clone, Debug)]
pub struct SimplicityTypedValue {
    pub(crate) inner: simplicityhl::Value,
}

impl SimplicityTypedValue {
    pub fn u8(value: u8) -> SimplicityTypedValue {
        SimplicityTypedValue {
            inner: simplicityhl::Value::u8(value),
        }
    }

    pub fn u16(value: u16) -> SimplicityTypedValue {
        SimplicityTypedValue {
            inner: simplicityhl::Value::u16(value),
        }
    }

    pub fn u32(value: u32) -> SimplicityTypedValue {
        SimplicityTypedValue {
            inner: simplicityhl::Value::u32(value),
        }
    }

    pub fn u64(value: u64) -> SimplicityTypedValue {
        SimplicityTypedValue {
            inner: simplicityhl::Value::u64(value),
        }
    }

    pub fn u128(bytes: Vec<u8>) -> anyhow::Result<SimplicityTypedValue, LwkError> {
        let arr: [u8; 16] = bytes
            .try_into()
            .map_err(|_| LwkError {
                msg: "u128 value must be 16 bytes".into(),
            })?;
        Ok(SimplicityTypedValue {
            inner: simplicityhl::Value::u128(u128::from_be_bytes(arr)),
        })
    }

    pub fn u256(bytes: Vec<u8>) -> anyhow::Result<SimplicityTypedValue, LwkError> {
        let arr: [u8; 32] = bytes
            .try_into()
            .map_err(|_| LwkError {
                msg: "u256 value must be 32 bytes".into(),
            })?;
        Ok(SimplicityTypedValue {
            inner: simplicityhl::Value::u256(U256::from_byte_array(arr)),
        })
    }

    pub fn boolean(value: bool) -> SimplicityTypedValue {
        SimplicityTypedValue {
            inner: simplicityhl::Value::from(value),
        }
    }

    pub fn left(value: &SimplicityTypedValue, right_type: &SimplicityType) -> SimplicityTypedValue {
        SimplicityTypedValue {
            inner: simplicityhl::Value::left(value.inner.clone(), right_type.inner().clone()),
        }
    }

    pub fn right(left_type: &SimplicityType, value: &SimplicityTypedValue) -> SimplicityTypedValue {
        SimplicityTypedValue {
            inner: simplicityhl::Value::right(left_type.inner().clone(), value.inner.clone()),
        }
    }

    pub fn tuple(elements: Vec<SimplicityTypedValue>) -> SimplicityTypedValue {
        SimplicityTypedValue {
            inner: simplicityhl::Value::tuple(elements.iter().map(|e| e.inner.clone())),
        }
    }

    pub fn none(inner_type: &SimplicityType) -> SimplicityTypedValue {
        SimplicityTypedValue {
            inner: simplicityhl::Value::none(inner_type.inner().clone()),
        }
    }

    pub fn some(value: &SimplicityTypedValue) -> SimplicityTypedValue {
        SimplicityTypedValue {
            inner: simplicityhl::Value::some(value.inner.clone()),
        }
    }

    pub fn byte_array(bytes: Vec<u8>) -> SimplicityTypedValue {
        SimplicityTypedValue {
            inner: simplicityhl::Value::byte_array(bytes.iter().copied()),
        }
    }

    pub fn parse(value_str: String, ty: &SimplicityType) -> anyhow::Result<SimplicityTypedValue, LwkError> {
        let inner = simplicityhl::Value::parse_from_str(&value_str, ty.inner())
            .map_err(|e| LwkError { msg: e.to_string() })?;
        Ok(SimplicityTypedValue { inner })
    }

    pub(crate) fn inner(&self) -> &simplicityhl::Value {
        &self.inner
    }
}
