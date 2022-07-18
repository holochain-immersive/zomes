use hdk::prelude::*;
use private_publication_integrity::{EntryTypes, PublicationRole};

#[hdk_extern]
pub fn assign_editor_role(assignee: AgentPubKey) -> ExternResult<()> {
    create_entry(&EntryTypes::PublicationRole(PublicationRole {
        role: String::from("editor"),
        assignee,
    }))?;

    Ok(())
}
