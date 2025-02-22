pub mod bot;
pub mod conversations;
pub mod interactions;
pub mod memories;
pub mod messages;
pub mod nodes;
pub mod state;

use crate::{Database, EngineError, MongoDbClient};

fn init_mongo_credentials() -> Option<mongodb::options::Credential> {
    let username = match std::env::var("MONGODB_USERNAME") {
        Ok(var) if var.len() > 0 => Some(var),
        _ => None,
    };
    let password = match std::env::var("MONGODB_PASSWORD") {
        Ok(var) if var.len() > 0 => Some(var),
        _ => None,
    };

    if let (&None, &None) = (&username, &password) {
        return None;
    }

    let credentials = mongodb::options::Credential::builder()
        .password(password)
        .username(username)
        .build();

    Some(credentials)
}

pub fn init() -> Result<Database, EngineError> {
    let hostname = match std::env::var("MONGODB_HOST") {
        Ok(var) => var,
        _ => panic!("Missing MONGODB_HOST in env"),
    };

    let dbname = match std::env::var("MONGODB_DATABASE") {
        Ok(var) => var,
        _ => panic!("Missing MONGODB_DATABASE in env"),
    };

    let port: Option<u16> = match std::env::var("MONGODB_PORT") {
        Ok(var) => match var.parse::<u16>() {
            Ok(port) => Some(port),
            Err(err) => return Err(EngineError::Manager(err.to_string())),
        },
        _ => None,
    };

    let credentials = init_mongo_credentials();

    let options = mongodb::options::ClientOptions::builder()
        .hosts(vec![mongodb::options::StreamAddress {
            hostname: hostname.into(),
            port,
        }])
        .credential(credentials)
        .build();

    let client = mongodb::sync::Client::with_options(options)?;
    let db = Database::Mongo(MongoDbClient::new(client.database(&dbname)));
    Ok(db)
}

pub fn get_db<'a>(db: &'a Database) -> Result<&'a MongoDbClient, EngineError> {
    match db {
        Database::Mongo(db) => Ok(db),
        _ => Err(EngineError::Manager(
            "MongoDB connector is not setup correctly".to_owned(),
        )),
    }
}

pub fn get_pagination_key(pagination_key: Option<String>) ->  Result<Option<String>, EngineError> {
    match pagination_key {
        Some(key) => {
            let base64decoded = match base64::decode(&key) {
                Ok(base64decoded) => base64decoded,
                Err(_) => return Err(EngineError::Manager(format!("Invalid pagination_key"))),
            };

            let key: String = match serde_json::from_slice(&base64decoded) {
                Ok(key) => key,
                Err(_) => return Err(EngineError::Manager(format!("Invalid pagination_key"))),
            };

            Ok(Some(key))
        },
        None => Ok(None)
    }
}