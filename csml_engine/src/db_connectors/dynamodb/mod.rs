use crate::data::DynamoDbClient;
use crate::{Client, Database, EngineError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use rusoto_dynamodb::AttributeValue;

pub mod aws_s3;
pub mod bot;
pub mod conversations;
pub mod interactions;
pub mod memories;
pub mod messages;
pub mod nodes;
pub mod state;
pub mod utils;

mod dynamodb_tests;

use crate::db_connectors::dynamodb::utils::*;

use rusoto_core::Region;

pub fn init() -> Result<Database, EngineError> {
    let region_name = std::env::var("AWS_REGION").ok();
    let dynamodb_endpoint = std::env::var("AWS_DYNAMODB_ENDPOINT").ok();
    let s3_endpoint = std::env::var("AWS_S3_ENDPOINT").ok();

    let mut dynamodb_region = Region::default();
    if let (Some(region_name), Some(dynamodb_endpoint)) = (region_name.clone(), dynamodb_endpoint) {
        dynamodb_region = Region::Custom {
            name: region_name,
            endpoint: dynamodb_endpoint,
        };
    }

    let mut s3_region = Region::default();
    if let (Some(region_name), Some(s3_endpoint)) = (region_name, s3_endpoint) {
        s3_region = Region::Custom {
            name: region_name,
            endpoint: s3_endpoint,
        };
    }

    // check that the table name is set in env
    get_table_name()?;

    let client = DynamoDbClient::new(dynamodb_region, s3_region);

    Ok(Database::Dynamodb(client))
}

pub fn get_db<'a>(db: &'a mut Database) -> Result<&'a mut DynamoDbClient, EngineError> {
    match db {
        Database::Dynamodb(val) => Ok(val),
        _ => Err(EngineError::Manager(
            "DynamoDB connector is not setup correctly".to_owned(),
        )),
    }
}

pub fn get_pagination_key(pagination_key: Option<String>) ->  Result<Option<HashMap<String, AttributeValue>>, EngineError> {
    match pagination_key {
        Some(key) => {
            let base64decoded = match base64::decode(&key) {
                Ok(base64decoded) => base64decoded,
                Err(_) => return Err(EngineError::Manager(format!("Invalid pagination_key"))),
            };
            match serde_json::from_slice(&base64decoded) {
                Ok(key) => Ok(Some(key)),
                Err(_) => return Err(EngineError::Manager(format!("Invalid pagination_key"))),
            }
        }
        None => Ok(None),
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DynamoDbKey {
    hash: String,
    range: String,
}

impl DynamoDbKey {
    pub fn new(hash: &str, range: &str) -> Self {
        Self {
            hash: hash.to_owned(),
            range: range.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bot {
    pub hash: String,
    pub range: String,
    pub range_time: String,
    pub class: String,

    pub version_id: String,
    pub id: String,
    pub bot: String,
    pub engine_version: String,
    pub created_at: String,
}

impl Bot {
    pub fn get_hash(id: &str) -> String {
        format!("bot#{}", id)
    }

    pub fn get_range(version_id: &str) -> String {
        make_range(&["version", version_id])
    }

    pub fn new(id: String, bot: String) -> Self {
        let version_id = Uuid::new_v4().to_string();
        let now = get_date_time();
        let version = env!("CARGO_PKG_VERSION");
        let class_name = "bot";

        Self {
            hash: Self::get_hash(&id),
            range: Self::get_range(&version_id),
            range_time: make_range(&[&class_name, &now, &version_id]),
            class: class_name.to_owned(),
            version_id,
            id,
            bot,
            engine_version: version.to_owned(),
            created_at: now,
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ConversationDeleteInfo {
    pub status: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Conversation {
    pub hash: String,
    pub range: String,
    pub range_time: String,
    pub class: String,
    pub id: String,
    pub client: Option<Client>,
    pub bot_id: Option<String>,
    pub channel_id: Option<String>,
    pub user_id: Option<String>,
    pub flow_id: String,
    pub step_id: String,
    pub status: String,
    pub last_interaction_at: String,
    pub updated_at: String,
    pub created_at: String,
}

impl Conversation {
    pub fn get_hash(client: &Client) -> String {
        make_hash(client)
    }

    pub fn get_range(status: &str, id: &str) -> String {
        make_range(&["conversation", status, id])
    }

    pub fn get_conversation_id_from_range(range: &str) -> String {
        let vec: Vec<&str> = range.split("#").collect();

        vec[2].to_owned()
    }

    pub fn get_key(client: &Client, status: &str, id: &str) -> DynamoDbKey {
        let hash = Self::get_hash(client);
        let range = Self::get_range(status, id);
        DynamoDbKey::new(&hash, &range)
    }

    /**
     * hash = bot_id:xxxx#channel_id:xxxx#user_id:xxxx
     * range = conversation#OPEN|CLOSED#id
     * range_time = conversation#OPEN|CLOSED#timestamp#id
     */
    pub fn new(client: &Client, flow_id: &str, step_id: &str) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = get_date_time();
        let status = "OPEN";
        let class_name = "conversation";
        Self {
            hash: Self::get_hash(client),
            range: Self::get_range("OPEN", &id),
            range_time: make_range(&[class_name, status, &now, &id]),
            id,
            client: Some(client.to_owned()),
            bot_id: Some(client.bot_id.to_owned()),
            channel_id: Some(client.channel_id.to_owned()),
            user_id: Some(client.user_id.to_owned()),
            class: class_name.to_owned(),
            flow_id: flow_id.to_owned(),
            step_id: step_id.to_owned(),
            status: status.to_owned(),
            last_interaction_at: now.to_owned(),
            updated_at: now.to_owned(),
            created_at: now.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InteractionDeleteInfo {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Interaction {
    pub hash: String,
    pub range: String,
    pub range_time: String,
    pub class: String,
    pub id: String,
    pub client: Option<Client>,
    pub bot_id: Option<String>,
    pub channel_id: Option<String>,
    pub user_id: Option<String>,
    pub success: bool,
    pub event: String,
    pub updated_at: String,
    pub created_at: String,
}

impl Interaction {
    pub fn get_hash(client: &Client) -> String {
        make_hash(client)
    }

    pub fn get_range(id: &str) -> String {
        make_range(&["interaction", id])
    }

    pub fn get_key(client: &Client, id: &str) -> DynamoDbKey {
        let hash = Self::get_hash(client);
        let range = Self::get_range(id);
        DynamoDbKey::new(&hash, &range)
    }

    /**
     * hash = bot_id:xxxx#channel_id:xxxx#user_id:xxxx
     * range = interaction#id
     * range_time = interaction#timestamp#id
     */
    pub fn new(id: &Uuid, client: &Client, encrypted_event: &str) -> Self {
        let class_name = "interaction";
        let now = get_date_time();
        let id = id.to_string();
        Self {
            hash: Self::get_hash(client),
            range: Self::get_range(&id),
            range_time: make_range(&[class_name, &now, &id]),
            class: class_name.to_string(),
            id: id.to_owned(),
            client: Some(client.clone()),
            bot_id: Some(client.bot_id.to_owned()),
            channel_id: Some(client.channel_id.to_owned()),
            user_id: Some(client.user_id.to_owned()),
            success: false,
            event: encrypted_event.to_owned(),
            updated_at: now.to_owned(),
            created_at: now.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemoryDeleteInfo {
    pub range: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MemoryGetInfo {
    pub key: String,
    pub value: Option<String>,
    pub created_at: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Memory {
    pub hash: String,
    pub range: String,
    pub range_time: String,
    pub class: String,
    pub client: Option<Client>,
    pub bot_id: Option<String>,
    pub channel_id: Option<String>,
    pub user_id: Option<String>,
    pub key: String,
    pub value: Option<String>,
    pub expires_at: Option<String>,
    pub created_at: String,
}

impl Memory {
    pub fn get_hash(client: &Client) -> String {
        make_hash(client)
    }

    pub fn get_range(key: &str) -> String {
        make_range(&["memory", key])
    }

    /**
     * hash = bot_id:xxxx#channel_id:xxxx#user_id:xxxx
     * range = memory#[mem_key]
     * range_time = memory#timestamp#[mem_key]
     */
    pub fn new(
        client: &Client,
        key: &str,
        encrypted_value: Option<String>,
    ) -> Self {
        let hash = Self::get_hash(client);
        let range = Self::get_range(key);
        let now = get_date_time();

        let class_name = "memory";
        Self {
            hash: hash.to_owned(),
            range: range.to_owned(),
            range_time: make_range(&[
                class_name,
                &now,
                &range
            ]),
            class: class_name.to_owned(),
            client: Some(client.to_owned()),
            bot_id: Some(client.bot_id.to_owned()),
            channel_id: Some(client.channel_id.to_owned()),
            user_id: Some(client.user_id.to_owned()),
            key: key.to_owned(),
            value: encrypted_value.clone(),
            expires_at: None,
            created_at: now.to_owned(),
        }
    }
}


#[derive(Serialize, Deserialize, Debug)]
struct MessageDeleteInfo {
    conversation_id: String,
    id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub hash: String,
    pub range: String,
    pub range_time: String,
    pub class: String,
    pub id: String,
    pub client: Option<Client>,
    pub bot_id: Option<String>,
    pub channel_id: Option<String>,
    pub user_id: Option<String>,
    pub interaction_id: String,
    pub conversation_id: String,
    pub flow_id: String,
    pub step_id: String,
    pub message_order: i32,
    pub interaction_order: i32,
    pub direction: String,
    pub payload: String,
    pub content_type: String,
    pub created_at: String,
}

impl Message {
    pub fn get_hash(client: &Client) -> String {
        make_hash(client)
    }

    pub fn get_range(conversation_id: &str, id: &str) -> String {
        make_range(&["message", conversation_id, id])
    }

    /**
     * hash = bot_id:xxxx#channel_id:xxxx#user_id:xxxx
     * range = message#conversation_id#id
     * range_time = message#timestamp#interaction_order#message_order#id
     */
    pub fn new(
        client: &Client,
        conversation_id: &str,
        interaction_id: &str,
        flow_id: &str,
        step_id: &str,
        direction: &str,
        interaction_order: i32,
        message_order: i32,
        payload: &str,
        content_type: &str,
    ) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let class_name = "message";
        let now = get_date_time();
        Self {
            hash: Self::get_hash(&client),
            range: Self::get_range(&conversation_id, &id),
            range_time: make_range(&[
                class_name,
                &now,
                &interaction_order.to_string(),
                &message_order.to_string(),
                &id,
            ]),
            class: class_name.to_owned(),
            id: id.to_owned(),
            client: Some(client.to_owned()),
            bot_id: Some(client.bot_id.to_owned()),
            channel_id: Some(client.channel_id.to_owned()),
            user_id: Some(client.user_id.to_owned()),
            interaction_id: interaction_id.to_owned(),
            conversation_id: conversation_id.to_owned(),
            flow_id: flow_id.to_owned(),
            step_id: step_id.to_owned(),
            message_order: message_order,
            interaction_order: interaction_order,
            direction: direction.to_owned(),
            payload: payload.to_owned(),
            content_type: content_type.to_owned(),
            created_at: now.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NodeDeleteInfo {
    pub id: String,
    pub conversation_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub hash: String,
    pub range: String,
    pub range_time: String,
    pub class: String,
    pub id: String,
    pub client: Option<Client>,
    pub bot_id: Option<String>,
    pub channel_id: Option<String>,
    pub user_id: Option<String>,
    pub conversation_id: String,
    pub interaction_id: String,
    pub flow_id: String,
    pub step_id: String,
    pub next_flow: Option<String>,
    pub next_step: Option<String>,
    pub created_at: String,
}

impl Node {
    /**
     * hash = conversation:xxxx
     * range = path#id
     * range_time = path#timestamp#id
     */
    pub fn new(
        client: &Client,
        conversation_id: &str,
        interaction_id: &str,
        flow_id: &str,
        step_id: &str,
        next_flow: Option<String>,
        next_step: Option<String>,
    ) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let class_name = "path";
        let now = get_date_time();
        Self {
            hash: make_range(&["conversation", conversation_id]),
            range: make_range(&[class_name, &id]),
            range_time: make_range(&[class_name, &now, &id]),
            class: class_name.to_owned(),
            id: id.to_owned(),
            client: Some(client.to_owned()),
            bot_id: Some(client.bot_id.to_owned()),
            channel_id: Some(client.channel_id.to_owned()),
            user_id: Some(client.user_id.to_owned()),
            conversation_id: conversation_id.to_owned(),
            interaction_id: interaction_id.to_owned(),
            flow_id: flow_id.to_owned(),
            step_id: step_id.to_owned(),
            next_flow: next_flow.clone(),
            next_step: next_step.clone(),
            created_at: now.to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatDeleteInfo {
    #[serde(rename = "type")]
    pub _type: String,
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    pub hash: String,
    pub range: String,
    pub class: String,
    pub id: String,
    pub client: Option<Client>,
    pub bot_id: Option<String>,
    pub channel_id: Option<String>,
    pub user_id: Option<String>,
    #[serde(rename = "type")]
    pub _type: String,
    pub key: String,
    pub value: String,
    pub expires_at: Option<String>,
    pub created_at: String,
}

impl State {
    pub fn get_hash(client: &Client) -> String {
        make_hash(client)
    }

    pub fn get_range(_type: &str, key: &str) -> String {
        make_range(&["state", _type, key])
    }

    /**
     * hash = bot_id:xxxx#channel_id:xxxx#user_id:xxxx
     * range = mem_state#id
     */
    pub fn new(client: &Client, _type: &str, key: &str, encrypted_value: &str) -> Self {
        let class_name = "state";
        let id = uuid::Uuid::new_v4().to_string();
        let now = get_date_time();
        Self {
            hash: Self::get_hash(&client),
            range: Self::get_range(_type, key),
            class: class_name.to_string(),
            id,
            client: Some(client.to_owned()),
            bot_id: Some(client.bot_id.to_owned()),
            channel_id: Some(client.channel_id.to_owned()),
            user_id: Some(client.user_id.to_owned()),
            _type: _type.to_string(),
            key: key.to_owned(),
            value: encrypted_value.to_owned(),
            expires_at: None,
            created_at: now.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Class {
    pub class: String,
    pub hash: String,
    pub range: String,
}