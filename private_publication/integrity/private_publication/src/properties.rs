use hdi::prelude::*;
use holo_hash::AgentPubKeyB64;

#[derive(Serialize, Deserialize, Debug, SerializedBytes)]
pub struct Properties {
    progenitor: AgentPubKeyB64,
}

#[hdk_extern]
pub fn progenitor(_: ()) -> ExternResult<AgentPubKey> {
    let properties = dna_info()?.properties;
    let progenitor_properties: Properties =
        Properties::try_from(properties).map_err(|err| wasm_error!(err))?;

    Ok(progenitor_properties.progenitor.into())
}
