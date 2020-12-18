#[cfg(feature = "dynamo")]
use crate::db_connectors::{dynamodb as dynamodb_connector, is_dynamodb};
#[cfg(feature = "dynamo")]
use csml_interpreter::data::csml_bot::DynamoBot;

#[cfg(feature = "mongo")]
use crate::db_connectors::{is_mongodb, mongodb as mongodb_connector};
use crate::error_messages::ERROR_DB_SETUP;
use crate::{CsmlBot, Database, EngineError};

pub fn create_bot_version(
    bot_id: String,
    csml_bot: CsmlBot,
    db: &mut Database,
) -> Result<String, EngineError> {
    #[cfg(feature = "mongo")]
    if is_mongodb() {
        let serializable_bot = csml_bot.to_serializable_bot();
        let bot = base64::encode(bincode::serialize(&serializable_bot).unwrap());

        let db = mongodb_connector::get_db(db)?;
        return mongodb_connector::bot::create_bot_version(bot_id, bot, db);
    }

    #[cfg(feature = "dynamo")]
    if is_dynamodb() {
        let db = dynamodb_connector::get_db(db)?;
        let flows = csml_bot.flows;

        let dynamo_bot = DynamoBot {
            id: csml_bot.id.to_owned(),
            name: csml_bot.name.to_owned(),
            custom_components: match csml_bot.custom_components.to_owned() {
                Some(value) => Some(value.to_string()),
                None => None
            },
            default_flow: csml_bot.default_flow.to_owned()
        };

        let bot = base64::encode(bincode::serialize(&dynamo_bot).unwrap());

        let id_bot = dynamodb_connector::bot::create_bot_version(bot_id.clone(), bot, db)?;
        dynamodb_connector::bot::create_flows_batches(bot_id, id_bot.clone(), flows, db)?;

        return Ok(id_bot)
    }

    Err(EngineError::Manager(ERROR_DB_SETUP.to_owned()))
}

pub fn get_last_bot_version(
    bot_id: &str,
    db: &mut Database,
) -> Result<Option<CsmlBot>, EngineError> {
    #[cfg(feature = "mongo")]
    if is_mongodb() {
        let db = mongodb_connector::get_db(db)?;
        return mongodb_connector::bot::get_last_bot_version(&bot_id, db);
    }

    #[cfg(feature = "dynamo")]
    if is_dynamodb() {
        let db = dynamodb_connector::get_db(db)?;
        return dynamodb_connector::bot::get_last_bot_version(&bot_id, db);
    }

    Err(EngineError::Manager(ERROR_DB_SETUP.to_owned()))
}

pub fn get_by_id(
    id: &str,
    _bot_id: &str,
    db: &mut Database,
) -> Result<Option<CsmlBot>, EngineError> {
    //HashMap<String, Flow>
    #[cfg(feature = "mongo")]
    if is_mongodb() {
        let db = mongodb_connector::get_db(db)?;
        return mongodb_connector::bot::get_bot_by_version_id(&id, db);
    }

    #[cfg(feature = "dynamo")]
    if is_dynamodb() {
        let db = dynamodb_connector::get_db(db)?;
        return dynamodb_connector::bot::get_bot_by_version_id(&id, &_bot_id, db);
    }

    Err(EngineError::Manager(ERROR_DB_SETUP.to_owned()))
}

pub fn get_bot_versions(
    bot_id: &str,
    last_key: Option<String>,
    db: &mut Database,
) -> Result<serde_json::Value, EngineError> {
    #[cfg(feature = "mongo")]
    if is_mongodb() {
        let db = mongodb_connector::get_db(db)?;
        return mongodb_connector::bot::get_bot_versions(&bot_id, last_key, db);
    }

    #[cfg(feature = "dynamo")]
    if is_dynamodb() {
        let db = dynamodb_connector::get_db(db)?;
        return dynamodb_connector::bot::get_bot_versions(&bot_id, last_key, db);
    }

    Err(EngineError::Manager(ERROR_DB_SETUP.to_owned()))
}