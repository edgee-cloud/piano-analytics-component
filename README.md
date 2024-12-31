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

This component implements the data collection protocol between [Edgee](https://www.edgee.cloud) and [Piano Analytics](https://developers.atinternet-solutions.com/piano-analytics/data-collection/how-to-send-events/collection-api).

## Quick Start

1. Download the latest component version from our [releases page](../../releases)
2. Place the `piano.wasm` file in your server (e.g., `/var/edgee/components`)
3. Add the following configuration to your `edgee.toml`:

```toml
[[destinations.data_collection]]
name = "piano"
component = "/var/edgee/components/piano.wasm"
credentials.piano_site_id = "..."
credentials.piano_collection_domain = "..."
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
[[destinations.data_collection]]
name = "piano"
component = "/var/edgee/components/piano.wasm"
credentials.piano_site_id = "..."
credentials.piano_collection_domain = "..."

# Optional configurations
config.anonymization = true        # Enable/disable data anonymization in case of pending or denied consent
config.default_consent = "pending" # Set default consent status if not specified by the user
```

### Event Controls
Control which events are forwarded to Piano Analytics:
```toml
config.page_event_enabled = true   # Enable/disable page event
config.track_event_enabled = true  # Enable/disable track event
config.user_event_enabled = true   # Enable/disable user event
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
make build
```

### Contributing
Interested in contributing? Read our [contribution guidelines](./CONTRIBUTING.md)

### Security
Report security vulnerabilities to [security@edgee.cloud](mailto:security@edgee.cloud)