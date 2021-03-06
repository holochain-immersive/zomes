use crate::properties::progenitor;
use hdi::prelude::*;

#[hdk_entry_helper]
pub struct PublicationRole {
    pub role: String,
    pub assignee: AgentPubKey,
}

pub fn validate_create_role(
    action: EntryCreationAction,
    role_entry: Entry,
) -> ExternResult<ValidateCallbackResult> {
    let publication_role = PublicationRole::try_from(role_entry)?;

    if publication_role.role != String::from("editor") {
        return Ok(ValidateCallbackResult::Invalid(
            "Only editor role is allowed".into(),
        ));
    }

    let progenitor_pub_key = progenitor()?;

    match action.author().eq(&progenitor_pub_key) {
        true => Ok(ValidateCallbackResult::Valid),
        false => Ok(ValidateCallbackResult::Invalid(
            "Only the progenitor can create roles".into(),
        )),
    }
}
