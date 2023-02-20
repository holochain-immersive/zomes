use hdi::prelude::*;
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
pub fn validate(_op: Op) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Valid)
}
