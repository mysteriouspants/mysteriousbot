use crate::ack_message_handler::AckMessageHandler;
use crate::mysterious_message_handler::MysteriousMessageHandler;
use crate::role_wizard::RoleWizard;
use crate::verbal_morality_handler::VerbalMoralityHandler;
use pickledb::{ PickleDb, PickleDbDumpPolicy, };
use toml::Value;
use toml::value::Table;


// this code is kinda unwrappy but I think that's okay because dying in
// initialization is sorta expected on bad config, right?
pub fn parse_handlers(
    raw_toml: String
) -> Vec<Box<dyn MysteriousMessageHandler>> {
    let toml = raw_toml.parse::<Value>().unwrap();
    let handlers = toml.as_table().unwrap().get("handlers").unwrap().as_array().unwrap();
    let mut parsed_handlers: Vec<Box<dyn MysteriousMessageHandler>> = Vec::new();

    for handler_value in handlers {
        let handler_config = handler_value.as_table().unwrap();
        let handler_type = handler_config.get("type").unwrap().as_str().unwrap();

        match handler_type {
            "AckMessage" => parsed_handlers.push(Box::new(
                ack_message_handler_from_config(&handler_config)
            )),
            "RoleWizard" => parsed_handlers.push(Box::new(
                role_wizard_from_config(&handler_config)
            )),
            "VerbalMorality" => parsed_handlers.push(Box::new(
                verbal_morality_handler_from_config(&handler_config)
            )),
            _ => { /* do nothing, i guess */ }
        };
    }

    parsed_handlers
}

fn role_wizard_from_config(config: &Table) -> RoleWizard {
    let grants = array_to_string_array(
        config.get("allowed_role_grants").unwrap().as_array().unwrap()
    );
    let revoke = array_to_string_array(
        config.get("allowed_role_revoke").unwrap().as_array().unwrap()
    );

    RoleWizard::new(grants, revoke)
}

fn ack_message_handler_from_config(config: &Table) -> AckMessageHandler {
    let deny_list = array_to_string_array(
        config.get("deny_channels").unwrap().as_array().unwrap()
    );

    AckMessageHandler::new(deny_list)
}

fn verbal_morality_handler_from_config(config: &Table) -> VerbalMoralityHandler {
    let bad_words: Vec<String> = array_to_string_array(
        config.get("bad_words").unwrap().as_array().unwrap()
    );
    let allow_users_by_tag: Vec<String> = array_to_string_array(
        config.get("allow_users_by_tag").unwrap().as_array().unwrap()
    );
    let deny_channels: Vec<String> = array_to_string_array(
        config.get("deny_channels").unwrap().as_array().unwrap()
    );
    let warning_message = config.get("warning_message")
        .unwrap().as_str().unwrap().to_owned();
    let db_name = config.get("counter_db_name").unwrap().as_str().unwrap();
    let db_path = format!("./db/{}", db_name);
    let infraction_counter = match PickleDb::load_json(&db_path, PickleDbDumpPolicy::AutoDump) {
        Ok(db) => db,
        Err(_) => PickleDb::new_json(&db_path, PickleDbDumpPolicy::AutoDump)
    };

    VerbalMoralityHandler::new(
        bad_words, allow_users_by_tag, deny_channels, warning_message,
        infraction_counter
    )
}

fn array_to_string_array(array: &Vec<Value>) -> Vec<String> {
    array.iter().map(|item| item.as_str().unwrap().to_owned()).collect()
}