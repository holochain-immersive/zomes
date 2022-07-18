use hdi::prelude::{*, holo_hash::AgentPubKeyB64};

#[hdk_extern]
pub fn validate(_op: Op) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}

#[derive(Serialize, Deserialize, Debug, SerializedBytes)]
pub struct Properties {
    progenitor: AgentPubKeyB64,
}

pub fn progenitor() -> ExternResult<AgentPubKey> {
    let properties = dna_info()?.properties;

    let progenitor_properties: Properties =
        Properties::try_from(properties).map_err(|err| wasm_error!(err.into()))?;

    Ok(progenitor_properties.progenitor.into())
}
