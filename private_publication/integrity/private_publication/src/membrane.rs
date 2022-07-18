use std::sync::Arc;

use hdi::prelude::*;
use membrane_proof::PrivatePublicationMembraneProof;

use crate::properties::progenitor;

pub fn is_membrane_proof_valid(
    for_agent: AgentPubKey,
    membrane_proof: Option<MembraneProof>,
) -> ExternResult<ValidateCallbackResult> {
    let progenitor_pub_key = progenitor()?;

    if for_agent == progenitor_pub_key {
        return Ok(ValidateCallbackResult::Valid);
    }

    match membrane_proof {
        None => Ok(ValidateCallbackResult::Invalid(
            "Invalid agent: no membrane proof present".into(),
        )),
        Some(proof) => {
            let bytes = Arc::try_unwrap(proof)
                .map_err(|err| wasm_error!(WasmErrorInner::Guest(format!("{:?}", err))))?;
            let record = Record::try_from(bytes).map_err(|err| wasm_error!(err.into()))?;

            if !record.action().author().eq(&progenitor_pub_key) {
                return Ok(ValidateCallbackResult::Invalid(
                    "The author of the record is not the progenitor".into(),
                ));
            }

            if !verify_signature(
                progenitor_pub_key,
                record.signature().clone(),
                record.action_hashed(),
            )? {
                return Ok(ValidateCallbackResult::Invalid(
                    "The signature of the record is not valid".into(),
                ));
            }

            let maybe_private_publication_membrane_proof: Option<PrivatePublicationMembraneProof> =
                record
                    .entry()
                    .to_app_option()
                    .map_err(|err| wasm_error!(err.into()))?;

            match maybe_private_publication_membrane_proof {
                Some(private_publication_membrane_proof) => {
                    if private_publication_membrane_proof.dna_hash != dna_info()?.hash {
                        return Ok(ValidateCallbackResult::Invalid(
                            "The membrane proof is not for this dna".into(),
                        ));
                    }

                    if !private_publication_membrane_proof.recipient.eq(&for_agent) {
                        return Ok(ValidateCallbackResult::Invalid(
                            "The membrane proof is not for this agent".into(),
                        ));
                    }
                }
                None => {
                    return Ok(ValidateCallbackResult::Invalid(
                        "Malformed membrane proof".into(),
                    ));
                }
            }

            Ok(ValidateCallbackResult::Valid)
        }
    }
}
