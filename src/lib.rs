mod piano_payload;

use crate::piano_payload::parse_value;
use exports::edgee::protocols::data_collection::Data;
use exports::edgee::protocols::data_collection::Dict;
use exports::edgee::protocols::data_collection::EdgeeRequest;
use exports::edgee::protocols::data_collection::Event;
use exports::edgee::protocols::data_collection::Guest;
use piano_payload::PianoEvent;
use piano_payload::PianoPayload;
use std::vec;
wit_bindgen::generate!({world: "data-collection", path: "wit", generate_all});

export!(PianoComponent);

struct PianoComponent;

impl Guest for PianoComponent {
    fn page(edgee_event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Page(ref data) = edgee_event.data {
            let mut payload =
                PianoPayload::new(&edgee_event, settings).map_err(|e| e.to_string())?;

            // page_view event
            let mut event =
                PianoEvent::new("page.display", &edgee_event).map_err(|e| e.to_string())?;

            event.data.pageview_id = Some(edgee_event.uuid.clone());
            if !data.name.is_empty() {
                event.data.page_name = Some(data.name.clone());
            }
            if !data.title.is_empty() {
                event.data.content_title = Some(data.title.clone());
                event.data.page_title_html = Some(data.title.clone());
                event.data.page = Some(data.title.clone());
            }
            event.data.content_keywords = Some(data.keywords.clone());
            event.data.event_url_full = Some(data.url.clone());
            event.data.previous_url = Some(data.referrer.clone());

            event.data.has_access = Some("anon".to_string());

            // add custom page properties
            if !data.properties.is_empty() {
                for (key, value) in data.properties.clone().iter() {
                    if key == "has_access" {
                        event.data.has_access = Some(value.clone());
                    } else {
                        event
                            .data
                            .additional_fields
                            .insert(key.clone(), parse_value(value));
                    }
                }
            }

            payload.events.push(event);

            Ok(build_edgee_request(payload))
        } else {
            Err("Missing page data".to_string())
        }
    }

    fn track(edgee_event: Event, settings: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Track(ref data) = edgee_event.data {
            if data.name.is_empty() {
                return Err("Missing event name".to_string());
            }

            let mut payload =
                PianoPayload::new(&edgee_event, settings).map_err(|e| e.to_string())?;

            // event
            let mut event =
                PianoEvent::new(data.name.as_str(), &edgee_event).map_err(|e| e.to_string())?;

            // add custom page properties
            if !data.properties.is_empty() {
                for (key, value) in data.properties.clone().iter() {
                    event
                        .data
                        .additional_fields
                        .insert(key.clone(), parse_value(value));
                }
            }

            payload.events.push(event);

            Ok(build_edgee_request(payload))
        } else {
            Err("Missing track data".to_string())
        }
    }

    fn user(_edgee_event: Event, _settings: Dict) -> Result<EdgeeRequest, String> {
        Err("User event not mapped to Piano Analytics".to_string())
    }
}

fn build_edgee_request(piano_payload: PianoPayload) -> EdgeeRequest {
    let mut headers = vec![];
    headers.push((String::from("content-type"), String::from("text/plain")));

    EdgeeRequest {
        method: exports::edgee::protocols::data_collection::HttpMethod::Post,
        url: format!(
            "https://{}/event?s={}&idclient={}",
            piano_payload.collection_domain, piano_payload.site_id, piano_payload.id_client
        ),
        headers,
        forward_client_headers: true,
        body: serde_json::to_string(&piano_payload).unwrap(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exports::edgee::protocols::data_collection::{
        Campaign, Client, Context, EventType, HttpMethod, PageData, Session, TrackData, UserData,
    };
    use exports::edgee::protocols::data_collection::Consent;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    fn sample_user_data(edgee_id: String) -> UserData {
        UserData {
            user_id: "123".to_string(),
            anonymous_id: "456".to_string(),
            edgee_id,
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
                ("user_category".to_string(), "whatever".to_string()),
            ],
        }
    }

    fn sample_context(
        edgee_id: String,
        locale: String,
        timezone: String,
        session_start: bool,
    ) -> Context {
        Context {
            page: sample_page_data(),
            user: sample_user_data(edgee_id),
            client: Client {
                city: "Paris".to_string(),
                ip: "192.168.0.1".to_string(),
                locale,
                timezone,
                user_agent: "Chrome".to_string(),
                user_agent_architecture: "fuck knows".to_string(),
                user_agent_bitness: "64".to_string(),
                user_agent_full_version_list: "Brand1;1.0.0|Brand2;2.0.0".to_string(),
                user_agent_version_list: "abc".to_string(),
                user_agent_mobile: "1".to_string(),
                user_agent_model: "don't know".to_string(),
                os_name: "MacOS".to_string(),
                os_version: "latest".to_string(),
                screen_width: 1024,
                screen_height: 768,
                screen_density: 2.0,
                continent: "Europe".to_string(),
                country_code: "FR".to_string(),
                country_name: "France".to_string(),
                region: "West Europe".to_string(),
            },
            campaign: Campaign {
                name: "random".to_string(),
                source: "random".to_string(),
                medium: "random".to_string(),
                term: "random".to_string(),
                content: "random".to_string(),
                creative_format: "random".to_string(),
                marketing_tactic: "random".to_string(),
            },
            session: Session {
                session_id: "random".to_string(),
                previous_session_id: "random".to_string(),
                session_count: 2,
                session_start,
                first_seen: 123,
                last_seen: 123,
            },
        }
    }

    fn sample_page_data() -> PageData {
        PageData {
            name: "page name".to_string(),
            category: "category".to_string(),
            keywords: vec!["value1".to_string(), "value2".into()],
            title: "page title".to_string(),
            url: "https://example.com/full-url?test=1".to_string(),
            path: "/full-path".to_string(),
            search: "?at_medium=abc&at_campaign=&at_something=true&at_something_else=false"
                .to_string(),
            referrer: "https://example.com/another-page".to_string(),
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
                ("has_access".to_string(), "true".to_string()),
            ],
        }
    }

    fn sample_page_event(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        timezone: String,
        session_start: bool,
    ) -> Event {
        Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::Page,
            data: Data::Page(sample_page_data()),
            context: sample_context(edgee_id, locale, timezone, session_start),
            consent,
        }
    }

    fn sample_track_data(event_name: String) -> TrackData {
        TrackData {
            name: event_name,
            products: vec![], // why is this mandatory?
            properties: vec![
                ("prop1".to_string(), "value1".to_string()),
                ("prop2".to_string(), "10".to_string()),
            ],
        }
    }

    fn sample_track_event(
        event_name: String,
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        timezone: String,
        session_start: bool,
    ) -> Event {
        Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::Track,
            data: Data::Track(sample_track_data(event_name)),
            context: sample_context(edgee_id, locale, timezone, session_start),
            consent,
        }
    }

    fn sample_user_event(
        consent: Option<Consent>,
        edgee_id: String,
        locale: String,
        timezone: String,
        session_start: bool,
    ) -> Event {
        Event {
            uuid: Uuid::new_v4().to_string(),
            timestamp: 123,
            timestamp_millis: 123,
            timestamp_micros: 123,
            event_type: EventType::User,
            data: Data::User(sample_user_data(edgee_id.clone())),
            context: sample_context(edgee_id, locale, timezone, session_start),
            consent,
        }
    }

    fn sample_collection_domain() -> String {
        "ABCDEFG.pa-cd.com".to_string()
    }

    fn sample_settings() -> Vec<(String, String)> {
        vec![
            ("piano_site_id".to_string(), "abc".to_string()),
            (
                "piano_collection_domain".to_string(),
                sample_collection_domain(),
            ),
        ]
    }

    #[test]
    fn page_with_consent() {
        let event = sample_page_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            "CET".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = PianoComponent::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert!(!edgee_request.body.is_empty());
        assert_eq!(
            edgee_request
                .url
                .starts_with(&format!("https://{}", sample_collection_domain())),
            true
        );
        // add more checks (headers, querystring, etc.)
    }

    #[test]
    fn page_without_consent() {
        let event = sample_page_event(
            None,
            "abc".to_string(),
            "fr".to_string(),
            "CET".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = PianoComponent::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert!(!edgee_request.body.is_empty());
    }

    #[test]
    fn page_with_dashed_locale() {
        let event = sample_page_event(
            None,
            "abc".to_string(),
            "fr-fr".to_string(),
            "CET".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = PianoComponent::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert!(!edgee_request.body.is_empty());
    }

    #[test]
    fn page_with_empty_locale() {
        let event = sample_page_event(
            None,
            Uuid::new_v4().to_string(),
            "".to_string(),
            "CET".to_string(),
            true,
        );

        let settings = sample_settings();
        let result = PianoComponent::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert!(!edgee_request.body.is_empty());
    }

    #[test]
    fn page_with_empty_timezone() {
        let event = sample_page_event(
            None,
            Uuid::new_v4().to_string(),
            "fr".to_string(),
            "".to_string(),
            true,
        );

        let settings = sample_settings();
        let result = PianoComponent::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert!(!edgee_request.body.is_empty());
    }

    #[test]
    fn page_not_session_start() {
        let event = sample_page_event(
            None,
            Uuid::new_v4().to_string(),
            "".to_string(),
            "CET".to_string(),
            false,
        );
        let settings = sample_settings();
        let result = PianoComponent::page(event, settings);

        assert_eq!(result.is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert!(!edgee_request.body.is_empty());
    }

    #[test]
    fn page_without_site_id_fails() {
        let event = sample_page_event(
            None,
            "abc".to_string(),
            "fr".to_string(),
            "CET".to_string(),
            true,
        );
        let settings: Vec<(String, String)> = vec![]; // empty
        let result = PianoComponent::page(event, settings); // this should panic!
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn page_without_collection_domain_fails() {
        let event = sample_page_event(
            None,
            "abc".to_string(),
            "fr".to_string(),
            "CET".to_string(),
            true,
        );
        let settings: Vec<(String, String)> = vec![
            ("piano_site_id".to_string(), "abc".to_string()), // only site ID
        ];
        let result = PianoComponent::page(event, settings); // this should panic!
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn track_with_consent() {
        let event = sample_track_event(
            "event-name".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            "CET".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = PianoComponent::track(event, settings);
        assert_eq!(result.clone().is_err(), false);
        let edgee_request = result.unwrap();
        assert_eq!(edgee_request.method, HttpMethod::Post);
        assert!(!edgee_request.body.is_empty());
    }

    #[test]
    fn track_with_empty_name_fails() {
        let event = sample_track_event(
            "".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            "CET".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = PianoComponent::track(event, settings);
        assert_eq!(result.is_err(), true);
    }

    #[test]
    fn user_event() {
        let event = sample_user_event(
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            "CET".to_string(),
            true,
        );
        let settings = sample_settings();
        let result = PianoComponent::user(event, settings);

        assert_eq!(result.clone().is_err(), true);
        assert_eq!(
            result
                .clone()
                .err()
                .unwrap()
                .to_string()
                .contains("not mapped"),
            true
        );
    }

    #[test]
    fn track_event_without_user_context_properties_and_empty_user_id() {
        let mut event = sample_track_event(
            "event-name".to_string(),
            Some(Consent::Granted),
            "abc".to_string(),
            "fr".to_string(),
            "CET".to_string(),
            true,
        );
        event.context.user.properties = vec![]; // empty context user properties
        event.context.user.user_id = "".to_string(); // empty context user id
        let settings = sample_settings();
        let result = PianoComponent::track(event, settings);
        //println!("Error: {}", result.clone().err().unwrap().to_string().as_str());
        assert_eq!(result.clone().is_err(), false);
    }
}
