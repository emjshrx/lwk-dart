use crate::contracts::simplicity::{SimplicityType, SimplicityTypedValue, SimplicityWitnessValues};

#[derive(Debug, Clone, Copy)]
pub(crate) enum IssuanceFactoryWitnessBranch {
    IssueAssets { output_index: u32 },
    RemoveFactory { output_index: u32 },
}

impl IssuanceFactoryWitnessBranch {
    pub(crate) fn build_witness_values(&self) -> SimplicityWitnessValues {
        let u32_type = SimplicityType::u32();
        let path_value = match self {
            IssuanceFactoryWitnessBranch::IssueAssets { output_index } => {
                SimplicityTypedValue::left(
                    &SimplicityTypedValue::u32(*output_index),
                    &u32_type,
                )
            }
            IssuanceFactoryWitnessBranch::RemoveFactory { output_index } => {
                SimplicityTypedValue::right(
                    &u32_type,
                    &SimplicityTypedValue::u32(*output_index),
                )
            }
        };

        SimplicityWitnessValues::new().add_value("PATH".into(), path_value)
    }
}
