use crate::data_contract::data_contract::DataContractV0;
use crate::data_contract::serialized_version::v0::DataContractInSerializationFormatV0;
use crate::data_contract::DataContract;
use crate::version::{FeatureVersion, PlatformVersion};
use crate::ProtocolError;
use bincode::{Decode, Encode};
use derive_more::From;
use platform_version::TryFromPlatformVersioned;

pub(in crate::data_contract) mod v0;

pub const CONTRACT_DESERIALIZATION_LIMIT: usize = 15000;

#[derive(Debug, Clone, Encode, Decode, From)]
pub enum DataContractInSerializationFormat {
    V0(DataContractInSerializationFormatV0),
}

impl TryFromPlatformVersioned<&DataContract> for DataContractInSerializationFormat {
    type Error = ProtocolError;

    fn try_from_platform_versioned(
        value: &DataContract,
        platform_version: &PlatformVersion,
    ) -> Result<Self, Self::Error> {
        match platform_version
            .dpp
            .contract_versions
            .contract_serialization_version
            .default_current_version
        {
            0 => {
                let v0_format: DataContractInSerializationFormatV0 = value.clone().into();
                Ok(v0_format.into())
            }
            version => Err(ProtocolError::UnknownVersionMismatch {
                method: "DataContract::serialize_to_default_current_version".to_string(),
                known_versions: vec![0],
                received: version,
            }),
        }
    }
}

impl TryFromPlatformVersioned<DataContract> for DataContractInSerializationFormat {
    type Error = ProtocolError;

    fn try_from_platform_versioned(
        value: DataContract,
        platform_version: &PlatformVersion,
    ) -> Result<Self, Self::Error> {
        match platform_version
            .dpp
            .contract_versions
            .contract_serialization_version
            .default_current_version
        {
            0 => {
                let v0_format: DataContractInSerializationFormatV0 = value.into();
                Ok(v0_format.into())
            }
            version => Err(ProtocolError::UnknownVersionMismatch {
                method: "DataContract::serialize_consume_to_default_current_version".to_string(),
                known_versions: vec![0],
                received: version,
            }),
        }
    }
}

impl TryFromPlatformVersioned<DataContractInSerializationFormat> for DataContract {
    type Error = ProtocolError;

    fn try_from_platform_versioned(
        value: DataContractInSerializationFormat,
        platform_version: &PlatformVersion,
    ) -> Result<Self, Self::Error> {
        match value {
            DataContractInSerializationFormat::V0(serialization_format_v0) => {
                match platform_version
                    .dpp
                    .contract_versions
                    .contract_structure_version
                {
                    0 => {
                        let data_contract = DataContractV0::try_from_platform_versioned(
                            serialization_format_v0,
                            platform_version,
                        )?;
                        Ok(data_contract.into())
                    }
                    version => Err(ProtocolError::UnknownVersionMismatch {
                        method: "DataContract::from_serialization_format".to_string(),
                        known_versions: vec![0],
                        received: version,
                    }),
                }
            }
        }
    }
}
