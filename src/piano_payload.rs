use crate::exports::provider::{Dict, Event};
use anyhow::anyhow;
use chrono::{TimeZone, Timelike, Utc};
use serde::Serialize;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Serialize, Debug, Default)]
pub(crate) struct PianoPayload {
    #[serde(skip)]
    pub site_id: String,
    #[serde(skip)]
    pub collection_domain: String,
    #[serde(skip)]
    pub id_client: String,
    pub(crate) events: Vec<PianoEvent>,
}

impl PianoPayload {
    pub(crate) fn new(edgee_event: &Event, cred_map: Dict) -> anyhow::Result<Self> {
        let cred: HashMap<String, String> = cred_map
            .iter()
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .collect();

        let site_id = match cred.get("piano_site_id") {
            Some(key) => key,
            None => return Err(anyhow!("Missing piano site id")),
        }
        .to_string();

        let collection_domain = match cred.get("piano_collection_domain") {
            Some(key) => key,
            None => return Err(anyhow!("Missing piano collection domain")),
        }
        .to_string();

        // todo: ID continuity
        let id_client = edgee_event.context.user.edgee_id.to_string();

        Ok(Self {
            site_id,
            collection_domain,
            id_client,
            events: vec![],
        })
    }
}

#[derive(Serialize, Debug, Default)]
pub(crate) struct PianoEvent {
    pub name: String,
    pub data: PianoData,
}
impl PianoEvent {
    pub(crate) fn new(name: &str, edgee_event: &Event) -> anyhow::Result<Self> {
        let mut event = PianoEvent::default();
        let mut data = PianoData::default();

        // Standard properties
        //
        // https://developers.atinternet-solutions.com/piano-analytics/data-collection/how-to-send-events/collection-api#standard-properties
        data.event_collection_platform = "edgee".to_string();
        data.event_collection_version = "1.0.0".to_string();

        // previous_url
        if !edgee_event.context.page.referrer.is_empty() {
            data.previous_url = Some(edgee_event.context.page.referrer.clone());
        }

        // Locale
        let locale = edgee_event.context.client.locale.clone();
        if locale.contains("-") {
            let parts: Vec<&str> = locale.split("-").collect();
            data.browser_language = parts[0].to_string();
            data.browser_language_local = parts[1].to_string();
        } else {
            data.browser_language = locale.clone();
            data.browser_language_local = locale.clone();
        }

        // device hour
        let timestamp = Utc.timestamp_opt(edgee_event.timestamp, 0);
        if !edgee_event.context.client.timezone.is_empty() {
            // this part uses the chrono_tz crate to convert the timestamp to the client timezone
            // unfortunately, the crate increase the weight of the binary by 0.9MB... this is huge
            let tz = chrono_tz::Tz::from_str(edgee_event.context.client.timezone.as_str());
            if tz.is_ok() {
                let tz = tz?;
                let dt = timestamp.unwrap().with_timezone(&tz);
                data.device_hour = dt.hour() as i64;
            }
        } else {
            data.device_hour = timestamp.unwrap().hour() as i64;
        }
        data.device_timestamp_utc = edgee_event.timestamp;
        data.device_local_hour = edgee_event.timestamp;

        // User Agent
        let ua_version = edgee_event.context.client.user_agent_version_list.clone();
        if !ua_version.is_empty() {
            data.ch_ua = string_to_ch_ua(&ua_version, false);
        }

        let ua_full_version = edgee_event
            .context
            .client
            .user_agent_full_version_list
            .clone();
        if !ua_full_version.is_empty() {
            data.ch_ua_full_version_list = string_to_ch_ua(&ua_full_version, true);
            if !data.ch_ua_full_version_list.is_empty() {
                data.ch_ua_full_version = data.ch_ua_full_version_list[0].version.clone();
            }
        }
        data.ch_ua_arch = edgee_event.context.client.user_agent_architecture.clone();
        data.ch_ua_bitness = edgee_event.context.client.user_agent_bitness.clone();
        let mut mobile = false;
        if !edgee_event.context.client.user_agent_mobile.is_empty()
            && edgee_event.context.client.user_agent_mobile == "1"
        {
            mobile = true;
        }
        data.ch_ua_mobile = mobile;
        data.ch_ua_model = edgee_event.context.client.user_agent_model.clone();
        data.ch_ua_platform = edgee_event.context.client.os_name.clone();
        data.ch_ua_platform_version = edgee_event.context.client.os_version.clone();

        // screen size
        if edgee_event.context.client.screen_width.is_positive() {
            data.device_display_width = edgee_event.context.client.screen_width as i64;
            data.device_screen_width = edgee_event.context.client.screen_width as i64;
        }
        if edgee_event.context.client.screen_height.is_positive() {
            data.device_display_height = edgee_event.context.client.screen_height as i64;
            data.device_screen_height = edgee_event.context.client.screen_height as i64;
        }

        // cookie_creation_date
        // get the first seen date from the session and convert it to a datetime string
        let first_seen_i64 = edgee_event.context.session.first_seen.clone();
        let first_seen_opt = chrono::DateTime::from_timestamp(first_seen_i64, 0);
        if let Some(first_seen) = first_seen_opt {
            data.cookie_creation_date = Some(first_seen.to_rfc3339());
        }

        // we set privacy consent to true and mode to optin because edgee is already handling privacy anonymization
        data.visitor_privacy_consent = true;
        data.visitor_privacy_mode = "optin".to_string();

        // Campaign
        //
        // We first use the standard campaign parameters coming from UTM, then we override them with the at_* parameters
        // get all at_* properties from edgee_event.context.page.search and add them to data.src_* properties
        // https://developers.atinternet-solutions.com/piano-analytics/data-collection/how-to-send-events/marketing-campaigns
        if !edgee_event.context.campaign.medium.is_empty() {
            data.src_medium = Some(edgee_event.context.campaign.medium.clone());
        }
        if !edgee_event.context.campaign.name.is_empty() {
            data.src_campaign = Some(edgee_event.context.campaign.name.clone());
        }
        if !edgee_event.context.campaign.source.is_empty() {
            data.src_source = Some(edgee_event.context.campaign.source.clone());
        }
        if !edgee_event.context.campaign.content.is_empty() {
            data.src_content = Some(edgee_event.context.campaign.content.clone());
        }
        if !edgee_event.context.campaign.term.is_empty() {
            data.src_term = Some(edgee_event.context.campaign.term.clone());
        }
        if !edgee_event.context.page.search.is_empty() {
            // analyze search string
            let qs = serde_qs::from_str(edgee_event.context.page.search.as_str());
            if qs.is_ok() {
                let qs_map: HashMap<String, String> = qs.unwrap();
                for (key, value) in qs_map.iter() {
                    if key.starts_with("at_") {
                        match key.as_str() {
                            "at_medium" => data.src_medium = Some(value.clone()),
                            "at_campaign" => data.src_campaign = Some(value.clone()),
                            _ => {
                                // replace at_ with src_
                                data.additional_fields
                                    .insert(key.replace("at_", "src_"), parse_value(value));
                            }
                        }
                    }
                }
            }
        }

        // User
        //
        // https://developers.atinternet-solutions.com/piano-analytics/data-collection/how-to-send-events/collection-api#users
        // https://developers.atinternet-solutions.com/piano-analytics/data-collection/how-to-send-events/users
        if !edgee_event.context.user.anonymous_id.is_empty() {
            data.user_id = Some(edgee_event.context.user.anonymous_id.clone());
        }
        if !edgee_event.context.user.user_id.is_empty() {
            data.user_id = Some(edgee_event.context.user.user_id.clone());
        }
        if !edgee_event.context.user.properties.is_empty() {
            for (key, value) in edgee_event.context.user.properties.clone().iter() {
                if key == "user_category" {
                    data.user_category = Some(value.clone());
                }
            }
        }

        event.name = name.to_string();
        event.data = data;
        Ok(event)
    }
}

pub fn parse_value(value: &str) -> serde_json::Value {
    if value == "true" {
        serde_json::Value::from(true)
    } else if value == "false" {
        serde_json::Value::from(false)
    } else if let Some(_v) = value.parse::<f64>().ok() {
        serde_json::Value::Number(value.parse().unwrap())
    } else {
        serde_json::Value::String(value.to_string())
    }
}

#[derive(Serialize, Debug, Default)]
pub(crate) struct PianoData {
    pub browser_language: String,
    pub browser_language_local: String,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ch_ua: Vec<ChUa>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ch_ua_arch: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ch_ua_bitness: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ch_ua_full_version: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ch_ua_full_version_list: Vec<ChUa>,
    pub ch_ua_mobile: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ch_ua_model: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ch_ua_platform: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub ch_ua_platform_version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_keywords: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cookie_creation_date: Option<String>,

    pub device_display_width: i64,
    pub device_display_height: i64,
    pub device_hour: i64,
    pub device_local_hour: i64,
    pub device_screen_width: i64,
    pub device_screen_height: i64,
    pub device_timestamp_utc: i64,

    pub event_collection_platform: String,
    pub event_collection_version: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_url_full: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_access: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_title_html: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pageview_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_url: Option<String>,

    pub visitor_privacy_consent: bool,
    pub visitor_privacy_mode: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_campaign: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_medium: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_term: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_category: Option<String>,

    #[serde(flatten)]
    pub additional_fields: HashMap<String, serde_json::Value>,
}

#[derive(Serialize, Debug, Default)]
pub(crate) struct ChUa {
    pub brand: String,
    pub version: String,
}

fn string_to_ch_ua(string: &str, full: bool) -> Vec<ChUa> {
    let mut ch_ua_list = vec![];
    if string.contains("|") {
        let parts: Vec<&str> = string.split("|").collect();
        for part in parts {
            if string.contains(";") {
                let parts: Vec<&str> = part.split(";").collect();
                if parts.len() != 2 {
                    continue;
                }
                let brand = parts[0];
                let mut version = parts[1];
                if !full && version.contains(".") {
                    let parts: Vec<&str> = version.split(".").collect();
                    version = parts[0];
                }
                ch_ua_list.push(ChUa {
                    brand: brand.to_string(),
                    version: version.to_string(),
                });
            }
        }
    } else {
        if string.contains(";") {
            let parts: Vec<&str> = string.split(";").collect();
            if parts.len() != 2 {
                return ch_ua_list;
            }
            let brand = parts[0];
            let mut version = parts[1];
            if !full && version.contains(".") {
                let parts: Vec<&str> = version.split(".").collect();
                version = parts[0];
            }
            ch_ua_list.push(ChUa {
                brand: brand.to_string(),
                version: version.to_string(),
            });
        }
    }
    ch_ua_list
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_to_chua_vec_single_entry() {
        let input = "Brand;1.0.0";
        let result = string_to_ch_ua(input, false);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].brand, "Brand");
        assert_eq!(result[0].version, "1");
    }

    #[test]
    fn string_to_chua_vec_multiple_entries() {
        let input = "Brand1;1.0.0|Brand2;2.0.0";
        let result = string_to_ch_ua(input, false);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].brand, "Brand1");
        assert_eq!(result[0].version, "1");
        assert_eq!(result[1].brand, "Brand2");
        assert_eq!(result[1].version, "2");
    }

    #[test]
    fn string_to_chua_vec_empty_string() {
        let input = "";
        let result = string_to_ch_ua(input, false);
        assert!(result.is_empty());
    }

    #[test]
    fn string_to_chua_vec_invalid_format() {
        let input = "InvalidFormat";
        let result = string_to_ch_ua(input, false);
        assert!(result.is_empty());
    }

    #[test]
    fn string_to_chua_vec_partial_invalid_format() {
        let input = "Brand1;1.0.0|InvalidFormat";
        let result = string_to_ch_ua(input, false);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].brand, "Brand1");
        assert_eq!(result[0].version, "1");
    }

    #[test]
    fn string_to_chua_vec_partial_invalid_format2() {
        let input = "Brand1;1,0.0|InvalidFormat";
        let result = string_to_ch_ua(input, false);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].brand, "Brand1");
        assert_eq!(result[0].version, "1,0");
    }

    #[test]
    fn string_to_chua_vec_partial_invalid_format3() {
        let input = "Brand1:1,0.0|Bar;foo";
        let result = string_to_ch_ua(input, false);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].brand, "Bar");
        assert_eq!(result[0].version, "foo");
    }

    #[test]
    fn string_to_chua_vec_partial_invalid_format4() {
        let input = "Brand1:1,0.0|Bar;foo|";
        let result = string_to_ch_ua(input, false);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].brand, "Bar");
        assert_eq!(result[0].version, "foo");
    }

    #[test]
    fn string_to_chua_vec_single_entry_full() {
        let input = "Brand;1.0.0";
        let result = string_to_ch_ua(input, true);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].brand, "Brand");
        assert_eq!(result[0].version, "1.0.0");
    }

    #[test]
    fn string_to_chua_vec_multiple_entries_full() {
        let input = "Brand1;1.0.0|Brand2;2.0.0";
        let result = string_to_ch_ua(input, true);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].brand, "Brand1");
        assert_eq!(result[0].version, "1.0.0");
        assert_eq!(result[1].brand, "Brand2");
        assert_eq!(result[1].version, "2.0.0");
    }
}
