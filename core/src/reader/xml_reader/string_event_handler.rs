use android_string::AndroidString;
use constants;
use reader::xml_reader::error::Error;
use reader::xml_reader::event_handler::EventHandler;
use reader::xml_reader::sinking_event_handler::SinkingEventHandler;
use xml::attribute::OwnedAttribute;

pub struct StringEventHandler {
    name: String,
    is_translatable: bool,
    built_android_string: Option<AndroidString>,
}

impl StringEventHandler {
    pub fn build(attributes: Vec<OwnedAttribute>) -> Result<StringEventHandler, Error> {
        let mut string_name = None;
        let mut is_translatable = true;
        for attribute in attributes {
            match attribute.name.local_name.as_str() {
                constants::attributes::NAME => string_name = Some(attribute.value),
                constants::attributes::TRANSLATABLE => {
                    if let constants::flags::FALSE = attribute.value.as_str() {
                        is_translatable = false
                    }
                }
                _ => {}
            }
        }

        match string_name {
            None => Err(Error::SyntaxError(String::from(
                "string element is missing required name attribute",
            ))),
            Some(name) => Ok(StringEventHandler {
                name,
                is_translatable,
                built_android_string: None,
            }),
        }
    }

    fn append_or_create_string(&mut self, text: String) {
        let text = match &self.built_android_string {
            None => text,
            Some(s) => format!("{}{}", s.value(), text),
        };

        self.built_android_string = Some(AndroidString::new(
            self.name.clone(),
            text,
            self.is_translatable,
        ));
    }
}

impl EventHandler for StringEventHandler {
    fn handler_for_start_element_event(
        &self,
        _tag_name: String,
        _attributes: Vec<OwnedAttribute>,
    ) -> Result<Box<EventHandler>, Error> {
        Ok(Box::new(SinkingEventHandler::new()))
    }

    fn handle_characters_event(&mut self, text: String) {
        self.append_or_create_string(text)
    }

    fn handle_cdata_event(&mut self, text: String) {
        self.append_or_create_string(format!("<![CDATA[{}]]>", text))
    }

    fn built_string(&self) -> Option<AndroidString> {
        self.built_android_string.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::StringEventHandler;
    use reader::xml_reader::event_handler::EventHandler;

    #[test]
    fn builds_string_with_one_character_event() {
        let mut handler = build_event_handler();
        handler.handle_characters_event(String::from("this is a test"));
        assert_eq!(handler.built_string().unwrap().value(), "this is a test")
    }

    #[test]
    fn builds_string_with_one_cdata_event() {
        let mut handler = build_event_handler();
        handler.handle_cdata_event(String::from("this is a test"));
        assert_eq!(
            handler.built_string().unwrap().value(),
            "<![CDATA[this is a test]]>"
        )
    }

    #[test]
    fn builds_string_with_character_followed_by_cdata_event() {
        let mut handler = build_event_handler();
        handler.handle_characters_event(String::from("character event "));
        handler.handle_cdata_event(String::from("cdata event"));
        assert_eq!(
            handler.built_string().unwrap().value(),
            "character event <![CDATA[cdata event]]>"
        )
    }

    #[test]
    fn builds_string_with_cdata_followed_by_character_event() {
        let mut handler = build_event_handler();
        handler.handle_cdata_event(String::from("cdata event"));
        handler.handle_characters_event(String::from(" character event"));
        assert_eq!(
            handler.built_string().unwrap().value(),
            "<![CDATA[cdata event]]> character event"
        )
    }

    #[test]
    fn builds_string_with_multiple_character_and_cdata_events() {
        let mut handler = build_event_handler();
        handler.handle_characters_event(String::from("character event 1 "));
        handler.handle_cdata_event(String::from("cdata event 1"));
        handler.handle_characters_event(String::from(" character event 2 "));
        handler.handle_cdata_event(String::from("cdata event 2"));
        handler.handle_characters_event(String::from(" "));
        handler.handle_cdata_event(String::from("cdata event 3"));
        handler.handle_characters_event(String::from(" character event 3"));
        assert_eq!(handler.built_string().unwrap().value(), "character event 1 <![CDATA[cdata event 1]]> character event 2 <![CDATA[cdata event 2]]> <![CDATA[cdata event 3]]> character event 3")
    }

    fn build_event_handler() -> StringEventHandler {
        StringEventHandler {
            name: String::from("test_string"),
            is_translatable: true,
            built_android_string: None,
        }
    }
}
