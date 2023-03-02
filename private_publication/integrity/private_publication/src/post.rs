use crate::{properties::progenitor, *};

#[derive(Clone)]
#[hdk_entry_helper]
pub struct Post {
    pub title: String,
    pub content: String,
}

pub fn validate_create_post(
    action: EntryCreationAction,
    post: Post,
) -> ExternResult<ValidateCallbackResult> {
    if action.author().eq(&progenitor(())?) {
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
    action: Update,
    post: Post,
    original_action: EntryCreationAction,
    original_post: Post,
) -> ExternResult<ValidateCallbackResult> {
    match original_action.author().eq(&action.author) {
        true => Ok(ValidateCallbackResult::Valid),
        false => Ok(ValidateCallbackResult::Invalid(String::from(
            "Only the author of a post can update it",
        ))),
    }
}
pub fn validate_delete_post(
    _action: Delete,
    _original_action: EntryCreationAction,
    _original_post: Post,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from(
        "Posts cannot be deleted",
    )))
}
pub fn validate_create_link_all_posts(
    _action: CreateLink,
    _base_address: AnyLinkableHash,
    target_address: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    let action_hash = ActionHash::from(target_address);
    let record = must_get_valid_record(action_hash)?;
    let _post: crate::Post = record
        .entry()
        .to_app_option()
        .map_err(|e| wasm_error!(e))?
        .ok_or(wasm_error!(WasmErrorInner::Guest(String::from(
            "Linked action must reference an entry"
        ))))?;
    Ok(ValidateCallbackResult::Valid)
}
pub fn validate_delete_link_all_posts(
    _action: DeleteLink,
    _original_action: CreateLink,
    _base: AnyLinkableHash,
    _target: AnyLinkableHash,
    _tag: LinkTag,
) -> ExternResult<ValidateCallbackResult> {
    Ok(ValidateCallbackResult::Invalid(String::from(
        "AllPosts links cannot be deleted",
    )))
}
