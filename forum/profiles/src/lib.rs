use hdk::prelude::*;

#[hdk_entry_helper]
pub struct Profile {
    pub nickname: String,
}

#[hdk_entry_defs]
#[unit_enum(UnitTypes)]
#[cfg(not(feature = "exercise1step1"))]
pub enum EntryTypes {
    #[entry_def(name = "profile")]
    Profile(Profile),
}

#[hdk_link_types]
#[cfg(not(feature = "exercise1step3"))]
pub enum LinkTypes {
    AgentToProfile,
}

// Create the given profile and associates it with our public key
#[hdk_extern]
#[cfg(not(feature = "exercise1step1"))]
pub fn create_profile(profile: Profile) -> ExternResult<ActionHash> {
    let action_hash = create_entry(EntryTypes::Profile(profile))?;

    let my_pub_key = agent_info()?.agent_initial_pubkey;

    create_link(
        my_pub_key,
        action_hash.clone(),
        LinkTypes::AgentToProfile,
        (),
    )?;

    Ok(action_hash)
}

// Gets the profile for the given agent, if they have created it
#[hdk_extern]
#[cfg(not(feature = "exercise1step4"))]
pub fn get_agent_profile(agent_pub_key: AgentPubKey) -> ExternResult<Option<Profile>> {
    inner_get_agent_profile(agent_pub_key)
}

// Gets the profile of the current agent, if we have created it
#[hdk_extern]
#[cfg(not(feature = "exercise1step5"))]
pub fn get_my_profile(_: ()) -> ExternResult<Option<Profile>> {
    let my_pub_key = agent_info()?.agent_initial_pubkey;

    inner_get_agent_profile(my_pub_key)
}

fn inner_get_agent_profile(agent_pub_key: AgentPubKey) -> ExternResult<Option<Profile>> {
    let links = get_links(
        agent_pub_key,
        LinkTypeFilter::single_type(zome_info()?.id, LinkType::new(0)),
        None,
    )?;

    match links.first() {
        Some(link) => get_profile(link.target.clone().into()),
        None => Ok(None),
    }
}

fn get_profile(action_hash: ActionHash) -> ExternResult<Option<Profile>> {
    let maybe_record = get(action_hash, GetOptions::default())?;

    match maybe_record {
        None => Ok(None),
        Some(record) => {
            let maybe_entry: Option<Entry> = record.entry.into_option();

            let entry: Entry = maybe_entry.ok_or(wasm_error!(WasmErrorInner::Guest(
                String::from("This record doesn't include any entry")
            )))?;

            let profile = Profile::try_from(entry)?;

            return Ok(Some(profile));
        }
    }
}
