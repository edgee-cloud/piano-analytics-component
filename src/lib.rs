mod piano_payload;

use crate::piano_payload::parse_value;
use exports::provider::{Data, Dict, EdgeeRequest, Event, Guest};
use piano_payload::PianoEvent;
use piano_payload::PianoPayload;
use std::vec;

wit_bindgen::generate!({world: "data-collection"});
export!(PianoComponent);

struct PianoComponent;

impl Guest for PianoComponent {
    fn page(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Page(ref data) = edgee_event.data {
            let mut payload =
                PianoPayload::new(&edgee_event, cred_map).map_err(|e| e.to_string())?;

            // page_view event
            let mut event =
                PianoEvent::new("page.display", &edgee_event).map_err(|e| e.to_string())?;

            event.data.pageview_id = Some(edgee_event.uuid.clone());
            event.data.page_name = Some(data.name.clone());
            event.data.content_title = Some(data.title.clone());
            event.data.page_title_html = Some(data.title.clone());
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

    fn track(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if let Data::Track(ref data) = edgee_event.data {
            if data.name.is_empty() {
                return Err("Missing event name".to_string());
            }

            let mut payload =
                PianoPayload::new(&edgee_event, cred_map).map_err(|e| e.to_string())?;

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

    fn user(edgee_event: Event, cred_map: Dict) -> Result<EdgeeRequest, String> {
        if let Data::User(ref data) = edgee_event.data {
            if data.user_id.is_empty() && data.anonymous_id.is_empty() {
                return Err("user_id or anonymous_id is not set".to_string());
            }

            let mut payload =
                PianoPayload::new(&edgee_event, cred_map).map_err(|e| e.to_string())?;

            // event
            let mut event = PianoEvent::new("identify", &edgee_event).map_err(|e| e.to_string())?;

            // User
            //
            // https://developers.atinternet-solutions.com/piano-analytics/data-collection/how-to-send-events/collection-api#users
            // https://developers.atinternet-solutions.com/piano-analytics/data-collection/how-to-send-events/users
            if !data.anonymous_id.is_empty() {
                event.data.user_id = Some(data.anonymous_id.clone());
            }
            if !data.user_id.is_empty() {
                event.data.user_id = Some(data.user_id.clone());
            }

            // add custom page properties
            if !data.properties.is_empty() {
                for (key, value) in data.properties.clone().iter() {
                    if key == "user_category" {
                        event.data.user_category = Some(value.clone());
                    } else {
                        event
                            .data
                            .additional_fields
                            .insert(key.clone(), parse_value(value));
                    }
                }
            }

            // add event to piano payload
            payload.events.push(event);

            Ok(build_edgee_request(payload))
        } else {
            Err("Missing user data".to_string())
        }
    }
}

fn build_edgee_request(piano_payload: PianoPayload) -> EdgeeRequest {
    let mut headers = vec![];
    headers.push((String::from("content-type"), String::from("text/plain")));

    EdgeeRequest {
        method: exports::provider::HttpMethod::Post,
        url: String::from(format!(
            "https://{}/event?s={}&idclient={}",
            piano_payload.collection_domain, piano_payload.site_id, piano_payload.id_client
        )),
        headers,
        body: serde_json::to_string(&piano_payload).unwrap(),
    }
}
