use hdk::prelude::{holo_hash::DnaHash, *};
use private_publication_lobby_integrity::{
    self, EntryTypes, LinkTypes, PrivatePublicationMembraneProof,
};

#[hdk_extern]
fn progenitor(_: ()) -> ExternResult<AgentPubKey> {
    private_publication_lobby_integrity::progenitor()
}

#[hdk_extern]
pub fn read_all_posts(_: ()) -> ExternResult<Vec<Record>> {
    let claims_records = query(
        ChainQueryFilter::new()
            .entry_type(EntryType::CapClaim)
            .include_entries(true),
    )?;

    let claims: Vec<CapClaim> = claims_records
        .into_iter()
        .filter_map(|record| record.entry().as_option().cloned())
        .filter_map(|entry| match entry {
            Entry::CapClaim(claim) => Some(claim.clone()),
            _ => None,
        })
        .collect();

    match claims.first() {
        None => Err(wasm_error!(WasmErrorInner::Guest(
            "We don't have capability to call this functions".into()
        ))),
        Some(claim) => {
            let response = call_remote(
                progenitor(())?,
                zome_info()?.name,
                "request_read_all_posts".into(),
                Some(claim.secret),
                (),
            )?;

            match response {
                ZomeCallResponse::Ok(result) => {
                    let posts: Vec<Record> = result.decode().map_err(|err| wasm_error!(err))?;

                    Ok(posts)
                }
                _ => Err(wasm_error!(WasmErrorInner::Guest(
                    "Error making the call remote".into()
                ))),
            }
        }
    }
}

#[hdk_extern]
pub fn request_read_all_posts(_: ()) -> ExternResult<Vec<Record>> {
    let response = call(
        CallTargetCell::OtherRole("private_publication".into()),
        ZomeName::from("posts"),
        "get_all_posts".into(),
        None,
        (),
    )?;

    match response {
        ZomeCallResponse::Ok(result) => {
            let posts: Vec<Record> = result.decode().map_err(|err| wasm_error!(err))?;

            Ok(posts)
        }
        _ => Err(wasm_error!(WasmErrorInner::Guest(
            "Error making the call remote".into()
        ))),
    }
}

fn build_secret() -> ExternResult<CapSecret> {
    let bytes = random_bytes(64)?;
    CapSecret::try_from(bytes.into_vec())
        .map_err(|_| wasm_error!(WasmErrorInner::Guest("Could not build secret".into())))
}

#[hdk_extern]
pub fn grant_capability_to_read(grantee: AgentPubKey) -> ExternResult<CapSecret> {
    let cap_secret = build_secret()?;

    let cap_grant_entry = CapGrantEntry {
        access: CapAccess::Assigned {
            secret: cap_secret.clone(),
            assignees: BTreeSet::from([grantee]),
        },
        functions: GrantedFunctions::Listed(BTreeSet::from([(
            zome_info()?.name,
            FunctionName::from("request_read_all_posts"),
        )])),
        tag: String::from(""),
    };

    create_cap_grant(cap_grant_entry)?;

    Ok(cap_secret)
}

#[hdk_extern]
pub fn store_capability_claim(cap_secret: CapSecret) -> ExternResult<()> {
    let cap_claim = CapClaim {
        grantor: progenitor(())?,
        secret: cap_secret,
        tag: String::from("get_all_posts"),
    };

    create_cap_claim(cap_claim)?;

    Ok(())
}

#[hdk_extern]
pub fn create_membrane_proof_for(agent_pub_key: AgentPubKey) -> ExternResult<()> {
    let response = call(
        CallTargetCell::OtherRole("private_publication".into()),
        ZomeName::from("posts"),
        "get_dna_hash".into(),
        None,
        (),
    )?;

    let hash: DnaHash = match response {
        ZomeCallResponse::Ok(result) => result.decode::<DnaHash>().map_err(|err| wasm_error!(err)),
        _ => Err(wasm_error!(WasmErrorInner::Guest(
            "Error making the call remote".into()
        ))),
    }?;

    let action_hash = create_entry(EntryTypes::PrivatePublicationMembraneProof(
        PrivatePublicationMembraneProof {
            dna_hash: hash,
            recipient: agent_pub_key.clone(),
        },
    ))?;

    create_link(
        agent_pub_key,
        action_hash,
        LinkTypes::AgentToMembraneProof,
        (),
    )?;

    Ok(())
}

#[hdk_extern]
pub fn get_my_membrane_proof(_: ()) -> ExternResult<Option<Record>> {
    let links = get_links(
        agent_info()?.agent_initial_pubkey,
        LinkTypes::AgentToMembraneProof,
        None,
    )?;

    match links.first() {
        None => Ok(None),
        Some(link) => get(ActionHash::from(link.target.clone()), GetOptions::default()),
    }
}
