pub mod data;
pub use csml_interpreter::{
    load_components,
    data::{
        ast::{Expr, Flow, InstructionScope},
        error_info::ErrorInfo,
        warnings::Warnings,
        Client, CsmlResult,
    }
};
use serde_json::json;

mod db_connectors;
mod error_messages;

mod encrypt;
mod init;
mod interpreter_actions;
mod send;
mod utils;

use data::*;
use db_connectors::{
    bot, memories, user, messages, conversations, init_db, state, BotVersion, BotVersionCreated,
    DbConversation,
};
use init::*;
use interpreter_actions::interpret_step;
use utils::*;

use csml_interpreter::data::{
    csml_bot::CsmlBot, csml_flow::CsmlFlow, Context, Hold, IndexInfo, Memory,
};
use std::{collections::HashMap, env, time::SystemTime};

/**
 * Initiate a CSML chat request.
 * Takes 2 arguments: the request being made and the CSML bot.
 * This method assumes that the bot is already validated in advance. A best practice is
 * to pre-validate the bot and store it in a valid state.
 *
 * The request must be made by a given client. Its unicity (used as a key for identifying
 * who made each new request and if they relate to an already-open conversation) is based
 * on a combination of 3 parameters that are assumed to be unique in their own context:
 * - bot_id: differentiate bots handled by the same CSML engine instance
 * - channel_id: a given bot may be used on different channels (messenger, slack...)
 * - user_id: differentiate users on the same communication channel
 */
pub fn start_conversation(
    request: CsmlRequest,
    bot_opt: BotOpt,
) -> Result<serde_json::Map<String, serde_json::Value>, EngineError> {
    let now = SystemTime::now();

    let formatted_event = format_event(json!(request))?;
    let mut db = init_db()?;

    let mut bot = bot_opt.search_bot(&mut db);
    init_bot(&mut bot)?;

    let mut data = init_conversation_info(
        get_default_flow(&bot)?.name.to_owned(),
        &formatted_event,
        &request,
        &bot,
        db,
    )?;

    // save event in db as message RECEIVE
    let msgs = vec![request.payload.to_owned()];
    messages::add_messages_bulk(&mut data, msgs, 0, "RECEIVE")?;

    check_for_hold(&mut data, &bot)?;

    let res = interpret_step(&mut data, formatted_event.to_owned(), &bot);

    if let Ok(var) = env::var(DEBUG) {
        if var == "true" {
            let el = now.elapsed()?;
            println!("Total time Manager - {}.{}", el.as_secs(), el.as_millis());
        }
    }
    res
}

/**
 * Return the latest conversation that is still open for a given user
 * (there should not be more than one), or None if there isn't any.
 */
pub fn get_open_conversation(client: &Client) -> Result<Option<DbConversation>, EngineError> {
    let mut db = init_db()?;

    conversations::get_latest_open(client, &mut db)
}


pub fn get_client_memories(client: &Client) -> Result<serde_json::Value, EngineError> {
    let mut db = init_db()?;

    memories::get_memories(client, &mut db)
}

pub fn get_client_memory(client: &Client, key: &str) -> Result<serde_json::Value, EngineError> {
    let mut db = init_db()?;

    memories::get_memory(client, key, &mut db)
}

pub fn get_client_messages(
    client: &Client,
    limit: Option<i64>,
    pagination_key: Option<String>,
) -> Result<serde_json::Value, EngineError> {
    let mut db = init_db()?;

    messages::get_client_messages(client, &mut db, limit, pagination_key)
}

pub fn get_client_conversations(
    client: &Client,
    limit: Option<i64>,
    pagination_key: Option<String>,
) -> Result<serde_json::Value, EngineError> {
    let mut db = init_db()?;

    conversations::get_client_conversations(client, &mut db, limit, pagination_key)
}

/**
 * Get current State ether Hold or NULL
 */
pub fn get_current_state(client: &Client) -> Result<Option<serde_json::Value>, EngineError> {
    let mut db = init_db()?;

    state::get_current_state(client, &mut db)
}

/**
 * Create memory
 */
pub fn create_client_memory(
    client: &Client,
    key: String,
    value: serde_json::Value,
) -> Result<(), EngineError> {
    let mut db = init_db()?;
    validate_memory_key_format(&key)?;

    memories::create_client_memory(client, key, value , &mut db)
}

/**
 * Create bot version
 */
pub fn create_bot_version(csml_bot: CsmlBot) -> Result<BotVersionCreated, EngineError> {
    let mut db = init_db()?;

    let bot_id = csml_bot.id.clone();

    match validate_bot(csml_bot.clone()) {
        CsmlResult {
            errors: Some(errors),
            ..
        } => Err(EngineError::Interpreter(format!("{:?}", errors))),
        CsmlResult { .. } => {
            let version_id = bot::create_bot_version(bot_id, csml_bot, &mut db)?;
            let engine_version = env!("CARGO_PKG_VERSION").to_owned();

            Ok(BotVersionCreated {
                version_id,
                engine_version,
            })
        }
    }
}

/**
 * get by bot_id
 */
pub fn get_last_bot_version(bot_id: &str) -> Result<Option<BotVersion>, EngineError> {
    let mut db = init_db()?;

    bot::get_last_bot_version(bot_id, &mut db)
}

/**
 * get bot by version_id
 */
pub fn get_bot_by_version_id(id: &str, bot_id: &str) -> Result<Option<BotVersion>, EngineError> {
    let mut db = init_db()?;

    bot::get_by_version_id(id, bot_id, &mut db)
}

/**
 * List the last 20 versions of the bot if no limit is set
 *
 * BOT = {
 *  "version_id": String,
 *  "id": String,
 *  "name": String,
 *  "custom_components": Option<String>,
 *  "default_flow": String
 *  "engine_version": String
 *  "created_at": String
 * }
 */
pub fn get_bot_versions(
    bot_id: &str,
    limit: Option<i64>,
    last_key: Option<String>,
) -> Result<serde_json::Value, EngineError> {
    let mut db = init_db()?;

    bot::get_bot_versions(bot_id, limit, last_key, &mut db)
}

/**
 * delete bot by version_id
 */
pub fn delete_bot_version_id(id: &str, bot_id: &str) -> Result<(), EngineError> {
    let mut db = init_db()?;

    bot::delete_bot_version(bot_id, id, &mut db)
}

/**
 * Delete all bot versions of bot_id
 */
pub fn delete_all_bot_versions(bot_id: &str) -> Result<(), EngineError> {
    let mut db = init_db()?;

    bot::delete_bot_versions(bot_id, &mut db)
}

/**
 * Delete all data related to bot: versions, conversations, messages, memories, nodes, integrations
 */
pub fn delete_all_bot_data(bot_id: &str) -> Result<(), EngineError> {
    let mut db = init_db()?;

    bot::delete_all_bot_data(bot_id, &mut db)
}

/**
 * Delete all the memories of a given client
 */
pub fn delete_client_memories(client: &Client) -> Result<(), EngineError> {
    let mut db = init_db()?;

    memories::delete_client_memories(client, &mut db)
}

/**
 * Delete a single memory for a given Client
 */
pub fn delete_client_memory(client: &Client, memory_name: &str,) -> Result<(), EngineError> {
    let mut db = init_db()?;

    memories::delete_client_memory(client, memory_name ,&mut db)
}

/**
 * Delete all data related to a given Client
 */
pub fn delete_client(client: &Client) -> Result<(), EngineError> {
    let mut db = init_db()?;

    user::delete_client(client, &mut db)
}

/**
 * List all the steps in every flow of a given CSML bot
 */
pub fn get_steps_from_flow(bot: CsmlBot) -> HashMap<String, Vec<String>> {
    csml_interpreter::get_steps_from_flow(bot)
}

/**
 * Simple static CSML bot linter.
 * Does not check for possible runtime errors, only for build-time errors
 * (missing steps or flows, syntax errors, etc.)
 */
pub fn validate_bot(mut bot: CsmlBot) -> CsmlResult {
    // load native components into the bot
    bot.native_components = match load_components() {
        Ok(components) => Some(components),
        Err(err) => return CsmlResult {
            errors: Some(vec!(err)),
            warnings: None,
            flows: None
        },
    };

    csml_interpreter::validate_bot(&bot)
}

/**
 * Close any open conversation a given client may currently have.
 * We also need to both clean the hold/local memory state to make sure
 * that outdated variables or hold positions are not loaded into the next open conversation.
 */
pub fn user_close_all_conversations(client: Client) -> Result<(), EngineError> {
    let mut db = init_db()?;

    state::delete_state_key(&client, "hold", "position", &mut db)?;
    conversations::close_all_conversations(&client, &mut db)
}

/**
 * Verify if the user is currently on hold in a given conversation.
 *
 * If a hold is found, make sure that the flow has not been updated since last conversation.
 * If that's the case, we can not be sure that the hold is in the same position,
 * so we need to clear the hold's position and restart the conversation.
 *
 * If the hold is valid, we also need to load the local step memory
 * (context.hold.step_vars) into the conversation context.
 */
fn check_for_hold(data: &mut ConversationInfo, bot: &CsmlBot) -> Result<(), EngineError> {
    match state::get_state_key(&data.client, "hold", "position", &mut data.db) {
        // user is currently on hold
        Ok(Some(hold)) => {
            match hold.get("hash") {
                Some(hash_value) => {
                    let flow_hash = get_current_step_hash(&data.context, bot)?;
                    // cleanup the current hold and restart flow
                    if flow_hash != *hash_value {
                        data.context.step = "start".to_owned();
                        return clean_hold_and_restart(data);
                    }
                    flow_hash
                }
                _ => return Ok(()),
            };

            let index = match serde_json::from_value::<IndexInfo>(hold["index"].clone()) {
                Ok(index) => index,
                Err(_) => {
                    state::delete_state_key(&data.client, "hold", "position", &mut data.db)?;
                    return Ok(());
                }
            };

            // all good, let's load the position and local variables
            data.context.hold = Some(Hold {
                index,
                step_vars: hold["step_vars"].clone(),
                step_name: data.context.step.to_owned(),
                flow_name: data.context.flow.to_owned(),
            });
           state::delete_state_key(&data.client, "hold", "position", &mut data.db)?;
        }
        // user is not on hold
        Ok(None) => (),
        Err(_) => (),
    };
    Ok(())
}
