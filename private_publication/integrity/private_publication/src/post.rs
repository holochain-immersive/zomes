use crate::{*, properties::progenitor};

#[hdk_entry_helper]
pub struct Post {
    pub title: String,
    pub content: String,
}

pub fn validate_create_post(action: EntryCreationAction) -> ExternResult<ValidateCallbackResult> {
    if action.author().eq(&progenitor()?) {
        return Ok(ValidateCallbackResult::Valid);
    }

    let role = PublicationRole {
        role: String::from("editor"),
        assignee: action.author().clone(),
    };

    let role_entry_hash = hash_entry(role)?;

    let _entry = must_get_entry(role_entry_hash)?;

    Ok(ValidateCallbackResult::Valid)
}

pub fn validate_update_post(
    original_action: EntryCreationAction,
    original_entry: Entry,
    action: Update,
    new_entry: Entry,
) -> ExternResult<ValidateCallbackResult> {
    match original_action.author().eq(&action.author) {
        true => Ok(ValidateCallbackResult::Valid),
        false => Ok(ValidateCallbackResult::Invalid(String::from(
            "Only the author of a post can update it",
        ))),
    }
}

pub fn validate_delete_post(
    original_action: EntryCreationAction,
    action: Delete,
) -> ExternResult<ValidateCallbackResult> {
    match original_action.author().eq(&action.author) {
        true => Ok(ValidateCallbackResult::Valid),
        false => Ok(ValidateCallbackResult::Invalid(String::from(
            "Only the author of a post can update it",
        ))),
    }
}
