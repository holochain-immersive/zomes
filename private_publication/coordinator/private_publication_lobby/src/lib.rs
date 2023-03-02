use hdk::prelude::{
    holo_hash::{DnaHash, DnaHashB64},
    *,
};
use private_publication_lobby_integrity::{
    self, EntryTypes, LinkTypes, PrivatePublicationMembraneProof,
};

fn build_secret() -> ExternResult<CapSecret> {
    let bytes = random_bytes(64)?;
    CapSecret::try_from(bytes.into_vec())
        .map_err(|_| wasm_error!(WasmErrorInner::Guest("Could not build secret".into())))
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GrantCapabilityToReadInput {
    reader: AgentPubKey,
    private_publication_dna_hash: DnaHash,
}
#[hdk_extern]
pub fn grant_capability_to_read(input: GrantCapabilityToReadInput) -> ExternResult<CapSecret> {
    let cap_secret = build_secret()?;

    let cap_grant_entry = CapGrantEntry {
        access: CapAccess::Assigned {
            secret: cap_secret.clone(),
            assignees: BTreeSet::from([input.reader]),
        },
        functions: GrantedFunctions::Listed(BTreeSet::from([(
            zome_info()?.name,
            FunctionName::from("request_read_private_publication_posts"),
        )])),
        tag: DnaHashB64::from(input.private_publication_dna_hash).to_string(),
    };

    create_cap_grant(cap_grant_entry)?;

    Ok(cap_secret)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StoreCapabilityClaimInput {
    cap_secret: CapSecret,
    author: AgentPubKey,
}

#[hdk_extern]
pub fn store_capability_claim(input: StoreCapabilityClaimInput) -> ExternResult<()> {
    let cap_claim = CapClaim {
        grantor: input.author,
        secret: input.cap_secret,
        tag: String::from("request_read_private_publication_posts"),
    };

    create_cap_claim(cap_claim)?;

    Ok(())
}

#[hdk_extern]
pub fn read_posts_for_author(author: AgentPubKey) -> ExternResult<Vec<Record>> {
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
        .filter(|claim| claim.grantor.eq(&author))
        .collect();

    match claims.first() {
        None => Err(wasm_error!(WasmErrorInner::Guest(
            "We don't have capability to call this functions".into()
        ))),
        Some(claim) => {
            let response = call_remote(
                author,
                zome_info()?.name,
                "request_read_private_publication_posts".into(),
                Some(claim.secret),
                (),
            )?;

            match response {
                ZomeCallResponse::Ok(result) => {
                    let posts: Vec<Record> = result.decode().map_err(|err| wasm_error!(err))?;

                    Ok(posts)
                }
                _ => Err(wasm_error!(WasmErrorInner::Guest(
                    format!("Error making the call remote {:?}", response)
                ))),
            }
        }
    }
}

#[hdk_extern]
pub fn request_read_private_publication_posts(_: ()) -> ExternResult<Vec<Record>> {
    let cap_grant = call_info()?.cap_grant;

    let CapGrant::RemoteAgent( zome_call_cap_grant) = cap_grant else {
        return Err(wasm_error!(WasmErrorInner::Guest(String::from("request_read_all_posts must be called using a cap grant"))));
    };

    let private_publication_dna_hash = DnaHash::from(
        DnaHashB64::from_b64_str(zome_call_cap_grant.tag.as_str()).or(Err(wasm_error!(
            WasmErrorInner::Guest(String::from("Bad cap_grant tag"))
        )))?,
    );

    let private_publication_cell_id = CellId::new(
        private_publication_dna_hash,
        agent_info()?.agent_latest_pubkey,
    );

    let response = call(
        CallTargetCell::OtherCell(private_publication_cell_id),
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
            format!("Error making the call remote {:?}", response)
        ))),
    }
}

/** Exercise 2 */

#[hdk_extern]
pub fn create_membrane_proof_for(proof: PrivatePublicationMembraneProof) -> ExternResult<()> {
    let action_hash = create_entry(EntryTypes::PrivatePublicationMembraneProof(proof.clone()))?;

    create_link(
        proof.recipient,
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
