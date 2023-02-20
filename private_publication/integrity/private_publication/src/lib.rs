#![allow(unused_variables)]
use hdi::prelude::*;

mod membrane;
mod post;
mod properties;
mod publication_role;

use membrane::is_membrane_proof_valid;
pub use post::*;
pub use publication_role::*;

#[hdk_entry_defs]
#[unit_enum(UnitEntryTypes)]
pub enum EntryTypes {
    Post(Post),
    PublicationRole(PublicationRole),
}

#[hdk_link_types]
pub enum LinkTypes {
    PathToPost,
}

////////////////////////////////////////////////////////////////////////////////
// Genesis self-check callback
////////////////////////////////////////////////////////////////////////////////
#[hdk_extern]
pub fn genesis_self_check(data: GenesisSelfCheckData) -> ExternResult<ValidateCallbackResult> {
    is_membrane_proof_valid(data.agent_key, data.membrane_proof)
}

pub fn validate_agent_joining(
    agent_pub_key: AgentPubKey,
    membrane_proof: &Option<MembraneProof>,
) -> ExternResult<ValidateCallbackResult> {
    is_membrane_proof_valid(agent_pub_key, membrane_proof.clone())
}

#[hdk_extern]
pub fn validate(op: Op) -> ExternResult<ValidateCallbackResult> {
    match op.to_type::<EntryTypes, LinkTypes>()? {
        OpType::StoreEntry(store_entry) => match store_entry {
            OpEntry::CreateEntry { app_entry, action } => match app_entry {
                EntryTypes::Post(post) => {
                    validate_create_post(EntryCreationAction::Create(action), post)
                }
                EntryTypes::PublicationRole(publication_role) => validate_create_publication_role(
                    EntryCreationAction::Create(action),
                    publication_role,
                ),
            },
            OpEntry::UpdateEntry {
                app_entry, action, ..
            } => match app_entry {
                EntryTypes::Post(post) => {
                    validate_create_post(EntryCreationAction::Update(action), post)
                }
                EntryTypes::PublicationRole(publication_role) => validate_create_publication_role(
                    EntryCreationAction::Update(action),
                    publication_role,
                ),
            },
            _ => Ok(ValidateCallbackResult::Valid),
        },
        OpType::RegisterUpdate(update_entry) => match update_entry {
            OpUpdate::Entry {
                original_action,
                original_app_entry,
                app_entry,
                action,
            } => match (app_entry, original_app_entry) {
                (
                    EntryTypes::PublicationRole(publication_role),
                    EntryTypes::PublicationRole(original_publication_role),
                ) => validate_update_publication_role(
                    action,
                    publication_role,
                    original_action,
                    original_publication_role,
                ),
                (EntryTypes::Post(post), EntryTypes::Post(original_post)) => {
                    validate_update_post(action, post, original_action, original_post)
                }
                _ => Ok(ValidateCallbackResult::Invalid(
                    "Original and updated entry types must be the same".to_string(),
                )),
            },
            _ => Ok(ValidateCallbackResult::Valid),
        },
        OpType::RegisterDelete(delete_entry) => match delete_entry {
            OpDelete::Entry {
                original_action,
                original_app_entry,
                action,
            } => match original_app_entry {
                EntryTypes::Post(post) => validate_delete_post(action, original_action, post),
                EntryTypes::PublicationRole(publication_role) => {
                    validate_delete_publication_role(action, original_action, publication_role)
                }
            },
            _ => Ok(ValidateCallbackResult::Valid),
        },
        OpType::RegisterCreateLink {
            link_type,
            base_address,
            target_address,
            tag,
            action,
        } => match link_type {
            LinkTypes::PathToPost => {
                validate_create_link_all_posts(action, base_address, target_address, tag)
            }
        },
        OpType::RegisterDeleteLink {
            link_type,
            base_address,
            target_address,
            tag,
            original_action,
            action,
        } => match link_type {
            LinkTypes::PathToPost => validate_delete_link_all_posts(
                action,
                original_action,
                base_address,
                target_address,
                tag,
            ),
        },
        OpType::StoreRecord(store_record) => match store_record {
            OpRecord::CreateEntry { app_entry, action } => match app_entry {
                EntryTypes::Post(post) => {
                    validate_create_post(EntryCreationAction::Create(action), post)
                }
                EntryTypes::PublicationRole(publication_role) => validate_create_publication_role(
                    EntryCreationAction::Create(action),
                    publication_role,
                ),
            },
            OpRecord::UpdateEntry {
                original_action_hash,
                app_entry,
                action,
                ..
            } => {
                let original_record = must_get_valid_record(original_action_hash)?;
                let original_action = original_record.action().clone();
                let original_action = match original_action {
                    Action::Create(create) => EntryCreationAction::Create(create),
                    Action::Update(update) => EntryCreationAction::Update(update),
                    _ => {
                        return Ok(ValidateCallbackResult::Invalid(
                            "Original action for an update must be a Create or Update action"
                                .to_string(),
                        ));
                    }
                };
                match app_entry {
                    EntryTypes::Post(post) => {
                        let result = validate_create_post(
                            EntryCreationAction::Update(action.clone()),
                            post.clone(),
                        )?;
                        if let ValidateCallbackResult::Valid = result {
                            let original_post: Option<Post> = original_record
                                .entry()
                                .to_app_option()
                                .map_err(|e| wasm_error!(e))?;
                            let original_post = match original_post {
                                Some(post) => post,
                                None => {
                                    return Ok(
                                            ValidateCallbackResult::Invalid(
                                                "The updated entry type must be the same as the original entry type"
                                                    .to_string(),
                                            ),
                                        );
                                }
                            };
                            validate_update_post(action, post, original_action, original_post)
                        } else {
                            Ok(result)
                        }
                    }
                    EntryTypes::PublicationRole(publication_role) => {
                        let result = validate_create_publication_role(
                            EntryCreationAction::Update(action.clone()),
                            publication_role.clone(),
                        )?;
                        if let ValidateCallbackResult::Valid = result {
                            let original_publication_role: Option<PublicationRole> =
                                original_record
                                    .entry()
                                    .to_app_option()
                                    .map_err(|e| wasm_error!(e))?;
                            let original_publication_role = match original_publication_role {
                                Some(publication_role) => publication_role,
                                None => {
                                    return Ok(
                                            ValidateCallbackResult::Invalid(
                                                "The updated entry type must be the same as the original entry type"
                                                    .to_string(),
                                            ),
                                        );
                                }
                            };
                            validate_update_publication_role(
                                action,
                                publication_role,
                                original_action,
                                original_publication_role,
                            )
                        } else {
                            Ok(result)
                        }
                    }
                }
            }
            OpRecord::DeleteEntry {
                original_action_hash,
                action,
                ..
            } => {
                let original_record = must_get_valid_record(original_action_hash)?;
                let original_action = original_record.action().clone();
                let original_action = match original_action {
                    Action::Create(create) => EntryCreationAction::Create(create),
                    Action::Update(update) => EntryCreationAction::Update(update),
                    _ => {
                        return Ok(ValidateCallbackResult::Invalid(
                            "Original action for a delete must be a Create or Update action"
                                .to_string(),
                        ));
                    }
                };
                let app_entry_type = match original_action.entry_type() {
                    EntryType::App(app_entry_type) => app_entry_type,
                    _ => {
                        return Ok(ValidateCallbackResult::Valid);
                    }
                };
                let entry = match original_record.entry().as_option() {
                    Some(entry) => entry,
                    None => {
                        if original_action.entry_type().visibility().is_public() {
                            return Ok(
                                    ValidateCallbackResult::Invalid(
                                        "Original record for a delete of a public entry must contain an entry"
                                            .to_string(),
                                    ),
                                );
                        } else {
                            return Ok(ValidateCallbackResult::Valid);
                        }
                    }
                };
                let original_app_entry = match EntryTypes::deserialize_from_type(
                    app_entry_type.zome_index.clone(),
                    app_entry_type.entry_index.clone(),
                    &entry,
                )? {
                    Some(app_entry) => app_entry,
                    None => {
                        return Ok(
                                ValidateCallbackResult::Invalid(
                                    "Original app entry must be one of the defined entry types for this zome"
                                        .to_string(),
                                ),
                            );
                    }
                };
                match original_app_entry {
                    EntryTypes::Post(original_post) => {
                        validate_delete_post(action, original_action, original_post)
                    }
                    EntryTypes::PublicationRole(original_publication_role) => {
                        validate_delete_publication_role(
                            action,
                            original_action,
                            original_publication_role,
                        )
                    }
                }
            }
            OpRecord::CreateLink {
                base_address,
                target_address,
                tag,
                link_type,
                action,
            } => match link_type {
                LinkTypes::PathToPost => {
                    validate_create_link_all_posts(action, base_address, target_address, tag)
                }
            },
            OpRecord::DeleteLink {
                original_action_hash,
                base_address,
                action,
            } => {
                let record = must_get_valid_record(original_action_hash)?;
                let create_link = match record.action() {
                    Action::CreateLink(create_link) => create_link.clone(),
                    _ => {
                        return Ok(ValidateCallbackResult::Invalid(
                            "The action that a DeleteLink deletes must be a CreateLink".to_string(),
                        ));
                    }
                };
                let link_type = match LinkTypes::from_type(
                    create_link.zome_index.clone(),
                    create_link.link_type.clone(),
                )? {
                    Some(lt) => lt,
                    None => {
                        return Ok(ValidateCallbackResult::Valid);
                    }
                };
                match link_type {
                    LinkTypes::PathToPost => validate_delete_link_all_posts(
                        action,
                        create_link.clone(),
                        base_address,
                        create_link.target_address,
                        create_link.tag,
                    ),
                }
            }
            OpRecord::CreatePrivateEntry { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::UpdatePrivateEntry { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::CreateCapClaim { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::CreateCapGrant { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::UpdateCapClaim { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::UpdateCapGrant { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::Dna { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::OpenChain { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::CloseChain { .. } => Ok(ValidateCallbackResult::Valid),
            OpRecord::InitZomesComplete { .. } => Ok(ValidateCallbackResult::Valid),
            _ => Ok(ValidateCallbackResult::Valid),
        },
        OpType::RegisterAgentActivity(agent_activity) => match agent_activity {
            OpActivity::CreateAgent { agent, action } => {
                let previous_action = must_get_action(action.prev_action)?;
                match previous_action.action() {
                        Action::AgentValidationPkg(
                            AgentValidationPkg { membrane_proof, .. },
                        ) => validate_agent_joining(agent, membrane_proof),
                        _ => {
                            Ok(
                                ValidateCallbackResult::Invalid(
                                    "The previous action for a `CreateAgent` action must be an `AgentValidationPkg`"
                                        .to_string(),
                                ),
                            )
                        }
                    }
            }
            _ => Ok(ValidateCallbackResult::Valid),
        },
    }
}
