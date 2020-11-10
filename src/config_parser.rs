use crate::ack_message_handler::AckMessageHandler;
use crate::mysterious_message_handler::MysteriousMessageHandler;
use crate::role_wizard::RoleWizard;
use toml::value::Table;
use toml::Value;

// this code is kinda unwrappy but I think that's okay because dying in
// initialization is sorta expected on bad config, right?
pub fn parse_handlers(raw_toml: String) -> Vec<Box<dyn MysteriousMessageHandler>> {
    let toml = raw_toml.parse::<Value>().unwrap();
    let handlers = toml
        .as_table()
        .unwrap()
        .get("handlers")
        .unwrap()
        .as_array()
        .unwrap();
    let mut parsed_handlers: Vec<Box<dyn MysteriousMessageHandler>> = Vec::new();

    for handler_value in handlers {
        let handler_config = handler_value.as_table().unwrap();
        let handler_type = handler_config.get("type").unwrap().as_str().unwrap();

        match handler_type {
            "AckMessage" => {
                parsed_handlers.push(Box::new(ack_message_handler_from_config(&handler_config)))
            }
            "RoleWizard" => {
                parsed_handlers.push(Box::new(role_wizard_from_config(&handler_config)))
            }
            _ => { /* do nothing, i guess */ }
        };
    }

    parsed_handlers
}

fn role_wizard_from_config(config: &Table) -> RoleWizard {
    let grants = array_to_string_array(
        config
            .get("allowed_role_grants")
            .unwrap()
            .as_array()
            .unwrap(),
    );
    let revoke = array_to_string_array(
        config
            .get("allowed_role_revoke")
            .unwrap()
            .as_array()
            .unwrap(),
    );

    RoleWizard::new(grants, revoke)
}

fn ack_message_handler_from_config(config: &Table) -> AckMessageHandler {
    let deny_list = array_to_string_array(config.get("deny_channels").unwrap().as_array().unwrap());

    AckMessageHandler::new(deny_list)
}

fn array_to_string_array(array: &Vec<Value>) -> Vec<String> {
    array
        .iter()
        .map(|item| item.as_str().unwrap().to_owned())
        .collect()
}
