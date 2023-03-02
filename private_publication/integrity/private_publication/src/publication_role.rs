use crate::properties::progenitor;
use hdi::prelude::*;

#[derive(Clone)]
#[hdk_entry_helper]
pub struct PublicationRole {
    pub role: String,
    pub assignee: AgentPubKey,
}

pub fn validate_create_publication_role(
    action: EntryCreationAction,
    publication_role: PublicationRole,
) -> ExternResult<ValidateCallbackResult> {
    if publication_role.role != String::from("editor") {
        return Ok(ValidateCallbackResult::Invalid(
            "Only editor role is allowed".into(),
        ));
    }

    let progenitor_pub_key = progenitor(())?;

    match action.author().eq(&progenitor_pub_key) {
        true => Ok(ValidateCallbackResult::Valid),
        false => Ok(ValidateCallbackResult::Invalid(
            "Only the progenitor can create roles".into(),
        )),
    }
}

pub fn validate_update_publication_role(
    _action: Update,
    _publication_role: PublicationRole,
    _original_action: EntryCreationAction,
    _original_publication_role: PublicationRole,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from(
        "Publication Roles cannot be updated",
    )))
}

pub fn validate_delete_publication_role(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_publication_role: PublicationRole,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from(
        "Publication Roles cannot be deleted",
    )))
}
