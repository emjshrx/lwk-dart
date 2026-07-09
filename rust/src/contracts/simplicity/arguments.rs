use std::collections::HashMap;

use lwk_simplicity::simplicityhl::parse::ParseFromStr;
use lwk_simplicity::simplicityhl::str::WitnessName;
use lwk_simplicity::simplicityhl::{Arguments, Value, WitnessValues};

use crate::api::error::LwkError;
use crate::contracts::simplicity::SimplicityTypedValue;

/// Builder for Simplicity program arguments.
#[derive(Clone, Default)]
pub struct SimplicityArguments {
    inner: HashMap<String, Value>,
}

impl SimplicityArguments {
    pub fn new() -> SimplicityArguments {
        SimplicityArguments::default()
    }

    pub fn add_value(&self, name: String, value: SimplicityTypedValue) -> SimplicityArguments {
        let mut new = self.clone();
        new.inner.insert(name, value.inner.clone());
        new
    }

    pub(crate) fn to_inner(&self) -> Result<Arguments, LwkError> {
        let map = self
            .inner
            .iter()
            .map(|(name, val)| {
                WitnessName::parse_from_str(name)
                    .map(|n| (n, val.clone()))
                    .map_err(|e| LwkError { msg: e.to_string() })
            })
            .collect::<Result<HashMap<_, _>, LwkError>>()?;
        Ok(Arguments::from(map))
    }
}

/// Builder for Simplicity witness values.
#[derive(Clone, Default)]
pub struct SimplicityWitnessValues {
    inner: HashMap<String, Value>,
}

impl SimplicityWitnessValues {
    pub fn new() -> SimplicityWitnessValues {
        SimplicityWitnessValues::default()
    }

    pub fn add_value(&self, name: String, value: SimplicityTypedValue) -> SimplicityWitnessValues {
        let mut new = self.clone();
        new.inner.insert(name, value.inner.clone());
        new
    }

    pub(crate) fn to_inner(&self) -> Result<WitnessValues, LwkError> {
        let map = self
            .inner
            .iter()
            .map(|(name, val)| {
                WitnessName::parse_from_str(name)
                    .map(|n| (n, val.clone()))
                    .map_err(|e| LwkError { msg: e.to_string() })
            })
            .collect::<Result<HashMap<_, _>, LwkError>>()?;
        Ok(WitnessValues::from(map))
    }
}
