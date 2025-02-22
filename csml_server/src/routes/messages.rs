use actix_web::{get, web, HttpResponse};
use csml_interpreter::data::{Client};
use serde::{Deserialize, Serialize};
use std::thread;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationIdPath {
    conversation_id: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetClientInfoQuery {
    user_id: String,
    bot_id: String,
    channel_id: String,
    limit: Option<i64>,
    pagination_key: Option<String>,
}

/**
 * List all the messages a client has ever exchanged with the chatbot
 */
#[get("/messages")]
pub async fn get_client_messages(query: web::Query<GetClientInfoQuery>) -> HttpResponse {

    let client = Client {
        bot_id: query.bot_id.to_owned(),
        channel_id: query.channel_id.to_owned(),
        user_id: query.user_id.to_owned()
    };

    let limit = query.limit.to_owned();
    let pagination_key = match query.pagination_key.to_owned() {
        Some(pagination_key) if pagination_key == "" => None,
        Some(pagination_key) => Some(pagination_key),
        None => None,
    };

    let res = thread::spawn(move || {
        csml_engine::get_client_messages(&client, limit, pagination_key)
    }).join().unwrap();

    match res {
        Ok(data) => HttpResponse::Ok().json(data),
        Err(err) => {
        eprintln!("EngineError: {:?}", err);
        HttpResponse::InternalServerError().finish()
        }
   }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use actix_web::http::{StatusCode};

    #[actix_rt::test]
    async fn test_messages() {
        let mut app = test::init_service(
            App::new()
                    .service(get_client_messages)
        ).await;

        let (user_id, channel_id, bot_id) = ("test", "messages-channel", "botid");

        let resp = test::TestRequest::get()
                    .uri(&format!("/messages?user_id={}&channel_id={}&bot_id={}", user_id, channel_id, bot_id))
                    .send_request(&mut app).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
}