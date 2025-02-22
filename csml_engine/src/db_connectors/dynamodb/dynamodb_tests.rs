#[cfg(test)]
mod tests {
    use crate::{
        Client,
        encrypt::encrypt_data,
        db_connectors::{
            init_db,
            dynamodb::{
                Message, get_db,
                messages::{write_messages_batch, delete_user_messages, get_client_messages},
                conversations::{create_conversation, get_client_conversations, delete_user_conversations},
                memories::{create_client_memory, internal_use_get_memories, 
                    delete_client_memories, delete_client_memory,
                    get_memories, get_memory
                }
            }
        }
    };

    fn get_client() -> Client {
        Client {
            user_id: "test".to_owned(),
            channel_id: "channel_id".to_owned(),
            bot_id: "bot_id".to_owned(),
        }

    }

    #[test]
    fn ok_messages() {
        let client = get_client();
        let conversation_id = "message_test";
        let mut db = init_db().unwrap();
        let db = get_db(&mut db).unwrap();

        delete_user_messages(&client, db).unwrap();

        let message = serde_json::json!({
            "content_type": "text",
            "content": "super message"
        });

        let messages = vec![
            Message::new(
                &client,
                conversation_id,
                "interaction_id",
                "Default",
                "start",
                "SEND",
                0,
                0,
                &encrypt_data(&message).unwrap(),
                "text",
            )
        ];

        write_messages_batch(&messages, db).unwrap();

        let response = get_client_messages(
            &client,
            db,
            Some(24),
            None
        ).unwrap();

        let received_msgs: Vec<serde_json::Value> = serde_json::from_value(response["messages"].clone()).unwrap();

        assert_eq!(messages.len(), received_msgs.len());

        assert_eq!(message, received_msgs[0]["payload"]);

        delete_user_messages(&client, db).unwrap();

        let response = get_client_messages(
            &client,
            db, 
            None, 
            None
        ).unwrap();

        let received_msgs: Vec<serde_json::Value> = serde_json::from_value(response["messages"].clone()).unwrap();

        assert_eq!(received_msgs.len(), 0)
    }

    #[test]
    fn ok_conversation() {
        let client = get_client();
        let mut db = init_db().unwrap();
        let db = get_db(&mut db).unwrap();

        delete_user_conversations(&client, db).unwrap();

        create_conversation("Default", "start", &client, db).unwrap();

        let response = get_client_conversations(&client, db, None, None).unwrap();
        let conversations: Vec<serde_json::Value> = serde_json::from_value(response["conversations"].clone()).unwrap();

        assert_eq!(conversations.len(), 1);

        assert_eq!("start", &conversations[0]["step_id"]);

        delete_user_conversations(&client, db).unwrap();

        let response = get_client_conversations(&client, db, None, None).unwrap();
        let conversations: Vec<serde_json::Value> = serde_json::from_value(response["conversations"].clone()).unwrap();
        assert_eq!(conversations.len(), 0);
    }

    #[test]
    fn ok_memories() {
        let client = get_client();
        let mut db = init_db().unwrap();
        let db = get_db(&mut db).unwrap();

        delete_client_memories(&client, db).unwrap();

        let mems = vec![
            ("key".to_owned(), serde_json::json!("value")),
            ("random".to_owned(), serde_json::json!(42)),
        ];

        for (key, value) in mems.iter() {
            create_client_memory(&client, key.to_owned(), value.to_owned(), db).unwrap();
        }

        let response = internal_use_get_memories(&client, db).unwrap();
        let memories: &serde_json::Map<String, serde_json::Value> = response.as_object().unwrap();

        assert_eq!(memories.len(), 2);

        for (key, value) in mems.iter() {
            assert_eq!(memories.get(key).unwrap(), value);
        }

        delete_client_memories(&client, db).unwrap();

        let response = internal_use_get_memories(&client, db).unwrap();
        let memories: &serde_json::Map<String, serde_json::Value> = response.as_object().unwrap();

        assert_eq!(memories.len(), 0);
    }

    #[test]
    fn ok_memory() {
        let client = get_client();
        let mut db = init_db().unwrap();
        let db = get_db(&mut db).unwrap();

        delete_client_memories(&client, db).unwrap();

        let mems = vec![
            ("memory_key".to_owned(), serde_json::json!("value")),
            ("memory".to_owned(), serde_json::json!("tmp")),
            ("memory_key".to_owned(), serde_json::json!("next")),
        ];

        for (key, value) in mems.iter() {
            create_client_memory(&client, key.to_owned(), value.to_owned(), db).unwrap();
        }

        let response = internal_use_get_memories(&client, db).unwrap();
        let memories: &serde_json::Map<String, serde_json::Value> = response.as_object().unwrap();

        assert_eq!(memories.len(), 2);

        let mems = vec![
            ("memory".to_owned(), serde_json::json!("tmp")),
            ("memory_key".to_owned(), serde_json::json!("next")),
        ];

        for (key, value) in mems.iter() {
            assert_eq!(memories.get(key).unwrap(), value);
        }

        delete_client_memory(&client, "memory", db).unwrap();

        let response = internal_use_get_memories(&client, db).unwrap();
        let memories: &serde_json::Map<String, serde_json::Value> = response.as_object().unwrap();

        assert_eq!(memories.len(), 1);

        let mems = vec![
            ("memory_key".to_owned(), serde_json::json!("next")),
        ];

        for (key, value) in mems.iter() {
            assert_eq!(memories.get(key).unwrap(), value);
        }
    }

    #[test]
    fn ok_get_memory() {
        let client = get_client();
        let mut db = init_db().unwrap();
        let db = get_db(&mut db).unwrap();

        delete_client_memories(&client, db).unwrap();

        let mems = vec![
            ("my_key".to_owned(), serde_json::json!("value")),
            ("random".to_owned(), serde_json::json!("tmp")),
            ("my_key".to_owned(), serde_json::json!("next")),
        ];

        for (key, value) in mems.iter() {
            create_client_memory(&client, key.to_owned(), value.to_owned(), db).unwrap();
        }

        let response = get_memory(&client, "my_key", db).unwrap();

        assert_eq!(serde_json::Value::String("next".to_owned()), response["value"]);


        let response = get_memories(&client, db).unwrap();

        match response {
            serde_json::Value::Array(memories) => {
                for memory in memories {
                    let key = memory["key"].as_str().unwrap();
                    if key != "random" && key != "my_key" {
                        panic!("bad memory => {:?}", memory)
                    }
                }
            }
            value => panic!("bad format => {:?}", value)
        }
    }
}