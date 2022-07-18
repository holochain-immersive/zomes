use hdi::prelude::{holo_hash::AgentPubKeyB64, *};
pub use membrane_proof::PrivatePublicationMembraneProof;

#[hdk_entry_defs]
#[unit_enum(UnitEntryType)]
pub enum EntryTypes {
    PrivatePublicationMembraneProof(PrivatePublicationMembraneProof),
}

#[hdk_link_types]
pub enum LinkTypes {
    AgentToMembraneProof,
}

#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
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
