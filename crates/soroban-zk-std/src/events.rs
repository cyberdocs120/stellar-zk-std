//! Structured event types for ZK proof verification.
//!
//! This module provides comprehensive event structures that developers can use
//! to understand exactly what happened during ZK proof verification, including
//! detailed failure information for debugging and monitoring.

use soroban_sdk::{
    contracttype, ConversionError, Env, EnvBase, MapObject, String, TryFromVal, TryIntoVal, Val,
};

use crate::ZkContractError;

/// The main event type published during ZK proof verification.
///
/// This structured event provides detailed information about verification
/// attempts, including success/failure status and specific failure details.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZkVerificationEvent {
    /// Whether the verification succeeded
    pub success: bool,
    /// The type of ZK proof system used
    pub proof_system: String,
    /// Number of public inputs provided
    pub public_inputs_count: u32,
    /// Length of the proof in bytes
    pub proof_length: u32,
    /// Whether anti-replay protection was enabled
    pub anti_replay: bool,
    /// Detailed failure information (None if successful)
    pub failure_details: Option<ZkVerificationFailure>,
    /// Additional context about the verification process
    pub context: Option<String>,
}

/// Detailed information about why a ZK proof verification failed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZkVerificationFailure {
    /// The specific error that caused the failure
    pub error_type: String,
    /// Human-readable description of the failure
    pub description: String,
    /// The stage of verification where the failure occurred
    pub failure_stage: ZkVerificationStage,
    /// Additional context specific to the failure type
    pub additional_info: Option<ZkFailureContext>,
}

/// Stages of ZK proof verification where failures can occur.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ZkVerificationStage {
    /// Failure during anti-replay check (before verification)
    AntiReplayCheck,
    /// Failure during verifying key loading
    VerifyingKeyLoad,
    /// Failure during proof deserialization
    ProofDeserialization,
    /// Failure during public inputs validation
    PublicInputsValidation,
    /// Failure during curve point validation
    CurveValidation,
    /// Failure during the pairing check computation
    PairingCheck,
    /// Failure during nullifier recording (after successful verification)
    NullifierRecording,
    /// Unknown/unexpected failure
    Unknown,
}

/// Additional context information for specific failure types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ZkFailureContext {
    /// For field element errors: the invalid input index
    pub invalid_input_index: Option<u32>,
    /// For deserialization errors: the problematic byte offset
    pub byte_offset: Option<u32>,
    /// For curve validation errors: which point failed (A, B, C for Groth16)
    pub invalid_point: Option<String>,
    /// For pairing errors: which pair in the multi-pairing failed
    pub invalid_pair_index: Option<u32>,
    /// Generic string context for other error types
    pub context_info: Option<String>,
}

impl ZkVerificationEvent {
    /// Creates a successful verification event.
    pub fn success(
        env: &Env,
        proof_system: &str,
        public_inputs_count: u32,
        proof_length: u32,
        anti_replay: bool,
    ) -> Self {
        Self {
            success: true,
            proof_system: String::from_str(env, proof_system),
            public_inputs_count,
            proof_length,
            anti_replay,
            failure_details: None,
            context: None,
        }
    }

    /// Creates a failure verification event with detailed error information.
    pub fn failure(
        env: &Env,
        proof_system: &str,
        public_inputs_count: u32,
        proof_length: u32,
        anti_replay: bool,
        error: &ZkContractError,
        stage: ZkVerificationStage,
        additional_info: Option<ZkFailureContext>,
    ) -> Self {
        let (error_type, description) = match error {
            ZkContractError::InvalidFieldElement => (
                "InvalidFieldElement",
                "One or more public inputs exceed the BN254 scalar field modulus",
            ),
            ZkContractError::InvalidInput => (
                "InvalidInput",
                "Invalid input parameters (wrong lengths, empty data, etc.)",
            ),
            ZkContractError::DeserializationError => (
                "DeserializationError",
                "Proof or point data could not be deserialized into valid structures",
            ),
            ZkContractError::HostError => {
                ("HostError", "Soroban host function call failed or trapped")
            }
            ZkContractError::StorageError => ("StorageError", "Ledger storage operation failed"),
            ZkContractError::ConstraintUnsatisfied => (
                "ConstraintUnsatisfied",
                "Zero-knowledge constraint or gadget invariant was violated",
            ),
            ZkContractError::ReplayDetected => (
                "ReplayDetected",
                "Proof has already been verified (anti-replay protection triggered)",
            ),
        };

        Self {
            success: false,
            proof_system: String::from_str(env, proof_system),
            public_inputs_count,
            proof_length,
            anti_replay,
            failure_details: Some(ZkVerificationFailure {
                error_type: String::from_str(env, error_type),
                description: String::from_str(env, description),
                failure_stage: stage,
                additional_info,
            }),
            context: None,
        }
    }

    /// Creates a failure event specifically for pairing check failures.
    pub fn pairing_failure(
        env: &Env,
        proof_system: &str,
        public_inputs_count: u32,
        proof_length: u32,
        anti_replay: bool,
    ) -> Self {
        Self {
            success: false,
            proof_system: String::from_str(env, proof_system),
            public_inputs_count,
            proof_length,
            anti_replay,
            failure_details: Some(ZkVerificationFailure {
                error_type: String::from_str(env, "PairingCheckFailed"),
                description: String::from_str(
                    env,
                    "The multi-pairing equation did not equal identity (proof is invalid)",
                ),
                failure_stage: ZkVerificationStage::PairingCheck,
                additional_info: Some(ZkFailureContext {
                    invalid_input_index: None,
                    byte_offset: None,
                    invalid_point: None,
                    invalid_pair_index: None,
                    context_info: Some(String::from_str(
                        env,
                        "e(A,B) * e(-alpha,beta) * e(-acc,gamma) * e(-C,delta) != 1",
                    )),
                }),
            }),
            context: None,
        }
    }

    /// Adds contextual information to the event.
    pub fn with_context(mut self, env: &Env, context: &str) -> Self {
        self.context = Some(String::from_str(env, context));
        self
    }
}

impl ZkFailureContext {
    /// Creates failure context for invalid field element at specific input index.
    pub fn invalid_input(index: u32) -> Self {
        Self {
            invalid_input_index: Some(index),
            byte_offset: None,
            invalid_point: None,
            invalid_pair_index: None,
            context_info: None,
        }
    }

    /// Creates failure context for deserialization error at specific byte offset.
    pub fn deserialization_error(byte_offset: u32) -> Self {
        Self {
            invalid_input_index: None,
            byte_offset: Some(byte_offset),
            invalid_point: None,
            invalid_pair_index: None,
            context_info: None,
        }
    }

    /// Creates failure context for invalid curve point.
    pub fn invalid_point(env: &Env, point_name: &str) -> Self {
        Self {
            invalid_input_index: None,
            byte_offset: None,
            invalid_point: Some(String::from_str(env, point_name)),
            invalid_pair_index: None,
            context_info: None,
        }
    }

    /// Creates failure context with generic string information.
    pub fn with_info(env: &Env, info: &str) -> Self {
        Self {
            invalid_input_index: None,
            byte_offset: None,
            invalid_point: None,
            invalid_pair_index: None,
            context_info: Some(String::from_str(env, info)),
        }
    }
}

// Implementations of TryFromVal for Soroban Env Val conversions

impl TryFromVal<Env, Val> for ZkVerificationEvent {
    type Error = ConversionError;
    fn try_from_val(env: &Env, val: &Val) -> Result<Self, ConversionError> {
        const KEYS: [&'static str; 7] = [
            "anti_replay",
            "context",
            "failure_details",
            "proof_length",
            "proof_system",
            "public_inputs_count",
            "success",
        ];
        let mut vals: [Val; 7] = [Val::VOID.to_val(); 7];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals)
            .map_err(|_| ConversionError)?;
        Ok(Self {
            anti_replay: vals[0].try_into_val(env)?,
            context: vals[1].try_into_val(env)?,
            failure_details: vals[2].try_into_val(env)?,
            proof_length: vals[3].try_into_val(env)?,
            proof_system: vals[4].try_into_val(env)?,
            public_inputs_count: vals[5].try_into_val(env)?,
            success: vals[6].try_into_val(env)?,
        })
    }
}

impl TryFromVal<Env, ZkVerificationEvent> for Val {
    type Error = ConversionError;
    fn try_from_val(env: &Env, val: &ZkVerificationEvent) -> Result<Self, ConversionError> {
        const KEYS: [&'static str; 7] = [
            "anti_replay",
            "context",
            "failure_details",
            "proof_length",
            "proof_system",
            "public_inputs_count",
            "success",
        ];
        let vals: [Val; 7] = [
            (&val.anti_replay)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.context)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.failure_details)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.proof_length)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.proof_system)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.public_inputs_count)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.success)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
        ];
        Ok(env
            .map_new_from_slices(&KEYS, &vals)
            .map_err(|_| ConversionError)?
            .into())
    }
}

impl TryFromVal<Env, &ZkVerificationEvent> for Val {
    type Error = ConversionError;
    #[inline(always)]
    fn try_from_val(env: &Env, val: &&ZkVerificationEvent) -> Result<Self, ConversionError> {
        Val::try_from_val(env, *val)
    }
}

impl TryFromVal<Env, Val> for ZkVerificationFailure {
    type Error = ConversionError;
    fn try_from_val(env: &Env, val: &Val) -> Result<Self, ConversionError> {
        const KEYS: [&'static str; 4] = [
            "additional_info",
            "description",
            "error_type",
            "failure_stage",
        ];
        let mut vals: [Val; 4] = [Val::VOID.to_val(); 4];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals)
            .map_err(|_| ConversionError)?;
        Ok(Self {
            additional_info: vals[0].try_into_val(env)?,
            description: vals[1].try_into_val(env)?,
            error_type: vals[2].try_into_val(env)?,
            failure_stage: vals[3].try_into_val(env)?,
        })
    }
}

impl TryFromVal<Env, ZkVerificationFailure> for Val {
    type Error = ConversionError;
    fn try_from_val(env: &Env, val: &ZkVerificationFailure) -> Result<Self, ConversionError> {
        const KEYS: [&'static str; 4] = [
            "additional_info",
            "description",
            "error_type",
            "failure_stage",
        ];
        let vals: [Val; 4] = [
            (&val.additional_info)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.description)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.error_type)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.failure_stage)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
        ];
        Ok(env
            .map_new_from_slices(&KEYS, &vals)
            .map_err(|_| ConversionError)?
            .into())
    }
}

impl TryFromVal<Env, &ZkVerificationFailure> for Val {
    type Error = ConversionError;
    #[inline(always)]
    fn try_from_val(env: &Env, val: &&ZkVerificationFailure) -> Result<Self, ConversionError> {
        Val::try_from_val(env, *val)
    }
}

impl TryFromVal<Env, Val> for ZkFailureContext {
    type Error = ConversionError;
    fn try_from_val(env: &Env, val: &Val) -> Result<Self, ConversionError> {
        const KEYS: [&'static str; 5] = [
            "byte_offset",
            "context_info",
            "invalid_input_index",
            "invalid_pair_index",
            "invalid_point",
        ];
        let mut vals: [Val; 5] = [Val::VOID.to_val(); 5];
        let map: MapObject = val.try_into().map_err(|_| ConversionError)?;
        env.map_unpack_to_slice(map, &KEYS, &mut vals)
            .map_err(|_| ConversionError)?;
        Ok(Self {
            byte_offset: vals[0].try_into_val(env)?,
            context_info: vals[1].try_into_val(env)?,
            invalid_input_index: vals[2].try_into_val(env)?,
            invalid_pair_index: vals[3].try_into_val(env)?,
            invalid_point: vals[4].try_into_val(env)?,
        })
    }
}

impl TryFromVal<Env, ZkFailureContext> for Val {
    type Error = ConversionError;
    fn try_from_val(env: &Env, val: &ZkFailureContext) -> Result<Self, ConversionError> {
        const KEYS: [&'static str; 5] = [
            "byte_offset",
            "context_info",
            "invalid_input_index",
            "invalid_pair_index",
            "invalid_point",
        ];
        let vals: [Val; 5] = [
            (&val.byte_offset)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.context_info)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.invalid_input_index)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.invalid_pair_index)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
            (&val.invalid_point)
                .try_into_val(env)
                .map_err(|_| ConversionError)?,
        ];
        Ok(env
            .map_new_from_slices(&KEYS, &vals)
            .map_err(|_| ConversionError)?
            .into())
    }
}

impl TryFromVal<Env, &ZkFailureContext> for Val {
    type Error = ConversionError;
    #[inline(always)]
    fn try_from_val(env: &Env, val: &&ZkFailureContext) -> Result<Self, ConversionError> {
        Val::try_from_val(env, *val)
    }
}

#[cfg(any(test, feature = "testutils"))]
impl TryFrom<&ZkFailureContext> for soroban_sdk::xdr::ScVal {
    type Error = soroban_sdk::xdr::Error;
    fn try_from(val: &ZkFailureContext) -> Result<Self, soroban_sdk::xdr::Error> {
        extern crate alloc;
        use soroban_sdk::xdr::{ScMap, ScMapEntry, ScSymbol, ScVal};
        ScMap::sorted_from(alloc::vec![
            ScMapEntry {
                key: ScSymbol(
                    "byte_offset"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.byte_offset)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
            ScMapEntry {
                key: ScSymbol(
                    "context_info"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.context_info)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
            ScMapEntry {
                key: ScSymbol(
                    "invalid_input_index"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.invalid_input_index)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
            ScMapEntry {
                key: ScSymbol(
                    "invalid_pair_index"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.invalid_pair_index)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
            ScMapEntry {
                key: ScSymbol(
                    "invalid_point"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.invalid_point)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
        ])
        .map(|m| ScVal::Map(Some(m)))
    }
}

#[cfg(any(test, feature = "testutils"))]
impl TryFrom<&ZkVerificationFailure> for soroban_sdk::xdr::ScVal {
    type Error = soroban_sdk::xdr::Error;
    fn try_from(val: &ZkVerificationFailure) -> Result<Self, soroban_sdk::xdr::Error> {
        extern crate alloc;
        use soroban_sdk::xdr::{ScMap, ScMapEntry, ScSymbol, ScVal};
        let add_info_scval = match &val.additional_info {
            Some(info) => ScVal::try_from(info)?,
            None => ScVal::Void,
        };
        ScMap::sorted_from(alloc::vec![
            ScMapEntry {
                key: ScSymbol(
                    "additional_info"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: add_info_scval
            },
            ScMapEntry {
                key: ScSymbol(
                    "description"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.description)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
            ScMapEntry {
                key: ScSymbol(
                    "error_type"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.error_type)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
            ScMapEntry {
                key: ScSymbol(
                    "failure_stage"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.failure_stage)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
        ])
        .map(|m| ScVal::Map(Some(m)))
    }
}

#[cfg(any(test, feature = "testutils"))]
impl TryFrom<&ZkVerificationEvent> for soroban_sdk::xdr::ScVal {
    type Error = soroban_sdk::xdr::Error;
    fn try_from(val: &ZkVerificationEvent) -> Result<Self, soroban_sdk::xdr::Error> {
        extern crate alloc;
        use soroban_sdk::xdr::{ScMap, ScMapEntry, ScSymbol, ScVal};
        let failure_details_scval = match &val.failure_details {
            Some(f) => ScVal::try_from(f)?,
            None => ScVal::Void,
        };
        ScMap::sorted_from(alloc::vec![
            ScMapEntry {
                key: ScSymbol(
                    "anti_replay"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.anti_replay)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
            ScMapEntry {
                key: ScSymbol(
                    "context"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.context)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
            ScMapEntry {
                key: ScSymbol(
                    "failure_details"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: failure_details_scval
            },
            ScMapEntry {
                key: ScSymbol(
                    "proof_length"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.proof_length)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
            ScMapEntry {
                key: ScSymbol(
                    "proof_system"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.proof_system)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
            ScMapEntry {
                key: ScSymbol(
                    "public_inputs_count"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.public_inputs_count)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
            ScMapEntry {
                key: ScSymbol(
                    "success"
                        .try_into()
                        .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
                )
                .into(),
                val: (&val.success)
                    .try_into()
                    .map_err(|_| soroban_sdk::xdr::Error::Invalid)?
            },
        ])
        .map(|m| ScVal::Map(Some(m)))
    }
}

/// Publishes a ZK verification event to the Soroban event system.
///
/// This is a convenience function that handles the event publishing with
/// appropriate topics for filtering and monitoring.
pub fn publish_verification_event(env: &Env, event: ZkVerificationEvent) {
    let topics = (
        String::from_str(env, "zk_verification"),
        event.proof_system.clone(),
        event.success,
    );

    env.events().publish(topics, event);
}

/// Legacy event publishing for backward compatibility.
///
/// Publishes a simple string-based event in the original format while also
/// emitting the new structured event.
#[allow(deprecated)]
pub fn publish_legacy_event(env: &Env, _success: bool, message: &str) {
    // Publish legacy format for backward compatibility
    env.events().publish(
        (String::from_str(env, "legacy"), String::from_str(env, "zk")),
        String::from_str(env, message),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{Env, String};

    #[test]
    fn test_successful_verification_event() {
        let env = Env::default();
        let event = ZkVerificationEvent::success(&env, "groth16", 2, 256, true);

        assert!(event.success);
        assert_eq!(event.proof_system, String::from_str(&env, "groth16"));
        assert_eq!(event.public_inputs_count, 2);
        assert_eq!(event.proof_length, 256);
        assert!(event.anti_replay);
        assert!(event.failure_details.is_none());
    }

    #[test]
    fn test_failure_verification_event() {
        let env = Env::default();
        let error = ZkContractError::InvalidFieldElement;
        let context = ZkFailureContext::invalid_input(1);

        let event = ZkVerificationEvent::failure(
            &env,
            "groth16",
            2,
            256,
            true,
            &error,
            ZkVerificationStage::PublicInputsValidation,
            Some(context),
        );

        assert!(!event.success);
        assert!(event.failure_details.is_some());

        let failure = event.failure_details.unwrap();
        assert_eq!(
            failure.error_type,
            String::from_str(&env, "InvalidFieldElement")
        );
        assert!(matches!(
            failure.failure_stage,
            ZkVerificationStage::PublicInputsValidation
        ));
        assert!(failure.additional_info.is_some());
        assert_eq!(
            failure.additional_info.unwrap().invalid_input_index,
            Some(1)
        );
    }

    #[test]
    fn test_pairing_failure_event() {
        let env = Env::default();
        let event = ZkVerificationEvent::pairing_failure(&env, "groth16", 1, 256, false);

        assert!(!event.success);
        assert!(event.failure_details.is_some());

        let failure = event.failure_details.unwrap();
        assert_eq!(
            failure.error_type,
            String::from_str(&env, "PairingCheckFailed")
        );
        assert!(matches!(
            failure.failure_stage,
            ZkVerificationStage::PairingCheck
        ));
    }

    #[test]
    fn test_event_with_context() {
        let env = Env::default();
        let event = ZkVerificationEvent::success(&env, "groth16", 1, 256, false)
            .with_context(&env, "Test verification");

        assert_eq!(
            event.context,
            Some(String::from_str(&env, "Test verification"))
        );
    }
}
