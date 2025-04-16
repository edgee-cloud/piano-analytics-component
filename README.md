<div align="center">
<p align="center">
  <a href="https://www.edgee.cloud">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://cdn.edgee.cloud/img/component-dark.svg">
      <img src="https://cdn.edgee.cloud/img/component.svg" height="100" alt="Edgee">
    </picture>
  </a>
</p>
</div>

<h1 align="center">Piano Analytics Component for Edgee</h1>

[![Coverage Status](https://coveralls.io/repos/github/edgee-cloud/piano-analytics-component/badge.svg)](https://coveralls.io/github/edgee-cloud/piano-analytics-component)
[![GitHub issues](https://img.shields.io/github/issues/edgee-cloud/piano-analytics-component.svg)](https://github.com/edgee-cloud/piano-analytics-component/issues)
[![Edgee Component Registry](https://img.shields.io/badge/Edgee_Component_Registry-Public-green.svg)](https://www.edgee.cloud/edgee/piano-analytics)

This component implements the data collection protocol between [Edgee](https://www.edgee.cloud) and [Piano Analytics](https://developers.atinternet-solutions.com/piano-analytics/data-collection/how-to-send-events/collection-api).

## Quick Start

1. Download the latest component version from our [releases page](../../releases)
2. Place the `piano.wasm` file in your server (e.g., `/var/edgee/components`)
3. Add the following configuration to your `edgee.toml`:

```toml
[[components.data_collection]]
id = "piano"
file = "/var/edgee/components/piano.wasm"
settings.piano_site_id = "..."
settings.piano_collection_domain = "..."
settings.piano_collect_utm_as_properties = "true"
```

## Event Handling

### Event Mapping
The component maps Edgee events to Piano Analytics events as follows:

| Edgee Event | Piano Analytics Event  | Description |
|-------------|----------------------- |-------------|
| Page        | `page.display`         | Triggered when a user views a page |
| Track       | Custom Event           | Uses the provided event name directly |
| User        | N/A                    | Used for user identification only |

### User Event Handling
While User events don't generate Piano Analytics events directly, they serve an important purpose:
- Stores `user_id`, `anonymous_id`, and `properties` on the user's device
- Enriches subsequent Page and Track events with user data
- Enables proper user attribution across sessions

## Configuration Options

### Basic Configuration
```toml
[[components.data_collection]]
id = "piano"
file = "/var/edgee/components/piano.wasm"
settings.piano_site_id = "..."
settings.piano_collection_domain = "..."
settings.piano_collect_utm_as_properties = "true"

# Optional configurations
settings.edgee_anonymization = true        # Enable/disable data anonymization in case of pending or denied consent
settings.edgee_default_consent = "pending" # Set default consent status if not specified by the user
```

To find out more about using `piano_collect_utm_as_properties`, please refer to [Piano documentation](https://developers.atinternet-solutions.com/piano-analytics/data-collection/how-to-send-events/marketing-campaigns#collect-utm-as-properties).

### Event Controls
Control which events are forwarded to Piano Analytics:
```toml
settings.edgee_page_event_enabled = true   # Enable/disable page event
settings.edgee_track_event_enabled = true  # Enable/disable track event
settings.edgee_user_event_enabled = true   # Enable/disable user event
```

### Consent Management
Before sending events to Google Analytics, you can set the user consent using the Edgee SDK: 
```javascript
edgee.consent("granted");
```

Or using the Data Layer:
```html
<script id="__EDGEE_DATA_LAYER__" type="application/json">
  {
    "data_collection": {
      "consent": "granted"
    }
  }
</script>
```

If the consent is not set, the component will use the default consent status.

| Consent | Anonymization | Piano Analytics Consent |
|---------|---------------|-------------------------|
| pending | true          | Exempt                  |
| denied  | true          | Exempt                  |
| granted | false         | Opt-in                  |

## Development

### Building from Source
Prerequisites:
- [Rust](https://www.rust-lang.org/tools/install)
- WASM target: `rustup target add wasm32-wasip2`
- wit-deps: `cargo install wit-deps`

Build command:
```bash
make wit-deps
make build
```

### Contributing
Interested in contributing? Read our [contribution guidelines](./CONTRIBUTING.md)

### Security
Report security vulnerabilities to [security@edgee.cloud](mailto:security@edgee.cloud)