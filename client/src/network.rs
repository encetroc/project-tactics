//! Handwritten SDK boundary. Presentation reads snapshots, never the SDK cache.
use std::sync::{Arc, Mutex, Weak};

use bevy::prelude::*;
use spacetimedb_sdk::{ConnectionId, DbContext, Table};

use crate::module_bindings::{DbConnection, PresenceTableAccess};

pub struct NetworkPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct NetworkSet;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ConnectionConfig::load())
            .init_resource::<Network>()
            .init_resource::<ConnectionSnapshot>()
            .add_systems(Startup, connect)
            .add_systems(Update, (reconnect, pump).chain().in_set(NetworkSet));
    }
}

#[derive(Resource)]
struct ConnectionConfig {
    server: String,
    database: String,
}

impl ConnectionConfig {
    fn load() -> Self {
        let mut config = Self {
            server: option_env!("SPACETIME_SERVER")
                .unwrap_or("ws://127.0.0.1:3001")
                .into(),
            database: option_env!("SPACETIME_DATABASE")
                .unwrap_or("project-tactics-dev")
                .into(),
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok(server) = std::env::var("SPACETIME_SERVER") {
                config.server = server;
            }
            if let Ok(database) = std::env::var("SPACETIME_DATABASE") {
                config.database = database;
            }
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(window) = web_sys::window()
            && let Ok(search) = window.location().search()
            && let Ok(params) = web_sys::UrlSearchParams::new_with_str(&search)
        {
            if let Some(server) = params.get("server") {
                config.server = server;
            }
            if let Some(database) = params.get("database") {
                config.database = database;
            }
        }
        config
    }
}

#[derive(Resource, Clone, PartialEq)]
pub struct ConnectionSnapshot {
    pub phase: &'static str,
    pub detail: String,
    pub sessions: Vec<ConnectionId>,
    pub local_session: Option<ConnectionId>,
}

impl Default for ConnectionSnapshot {
    fn default() -> Self {
        Self {
            phase: "connecting",
            detail: "Opening connection…".into(),
            sessions: Vec::new(),
            local_session: None,
        }
    }
}

impl ConnectionSnapshot {
    pub fn summary(&self) -> String {
        let mut summary = format!("Status: {}\n{}\n", self.phase, self.detail);
        if self.phase != "connected" {
            summary.push_str("Connected clients: unavailable\nPress R or reload to reconnect.");
            return summary;
        }
        summary.push_str(&format!("Connected clients: {}\n", self.sessions.len()));
        for session in &self.sessions {
            let marker = if Some(*session) == self.local_session {
                " (you)"
            } else {
                ""
            };
            summary.push_str(&format!("{session}{marker}\n"));
        }
        summary
    }
}

type Inbox = Arc<Mutex<Vec<NetworkEvent>>>;
type EventSink = Weak<Mutex<Vec<NetworkEvent>>>;

enum NetworkEvent {
    Built(Result<DbConnection, String>),
    Ready,
    Failed(String),
    Disconnected(String),
}

#[derive(Resource, Default)]
struct Network {
    connection: Option<DbConnection>,
    inbox: Inbox,
}

fn send(inbox: &EventSink, event: NetworkEvent) {
    if let Some(inbox) = inbox.upgrade() {
        inbox.lock().expect("network inbox poisoned").push(event);
    } else if let NetworkEvent::Built(Ok(connection)) = event {
        // A pending browser build may finish after the attempt was abandoned.
        let _ = connection.disconnect();
    }
}

fn connect(
    mut network: ResMut<Network>,
    config: Res<ConnectionConfig>,
    mut snapshot: ResMut<ConnectionSnapshot>,
) {
    begin_connection(&mut network, &config, &mut snapshot);
}

fn validate_endpoint(server: &str) -> Result<http::Uri, String> {
    let uri: http::Uri = server
        .parse()
        .map_err(|_| "Invalid server URL".to_string())?;
    if !matches!(uri.scheme_str(), Some("ws" | "wss" | "http" | "https")) || uri.host().is_none() {
        return Err("Server URL must include an http(s):// or ws(s):// host".into());
    }
    Ok(uri)
}

fn begin_connection(
    network: &mut Network,
    config: &ConnectionConfig,
    snapshot: &mut ConnectionSnapshot,
) {
    if let Some(connection) = network.connection.take() {
        let _ = connection.disconnect();
    }
    // Isolate callback generations so an old socket cannot change the new snapshot.
    network.inbox = Inbox::default();
    *snapshot = ConnectionSnapshot {
        detail: format!("{} / {}", config.server, config.database),
        ..default()
    };
    let uri = match validate_endpoint(&config.server) {
        Ok(uri) => uri,
        Err(error) => {
            snapshot.phase = "error";
            snapshot.detail = error;
            return;
        }
    };
    let ready = Arc::downgrade(&network.inbox);
    let subscription_error = Arc::downgrade(&network.inbox);
    let connect_error = Arc::downgrade(&network.inbox);
    let disconnected = Arc::downgrade(&network.inbox);
    let builder = DbConnection::builder()
        .with_uri(uri)
        .with_database_name(&config.database)
        .on_connect(move |connection, _, _token| {
            // Tokens are intentionally neither persisted nor logged.
            connection
                .subscription_builder()
                .on_applied(move |_| send(&ready, NetworkEvent::Ready))
                .on_error(move |_, error| {
                    send(&subscription_error, NetworkEvent::Failed(error.to_string()))
                })
                .subscribe(["SELECT * FROM presence"]);
        })
        .on_connect_error(move |_, error| {
            send(&connect_error, NetworkEvent::Failed(error.to_string()))
        })
        .on_disconnect(move |_, error| {
            send(
                &disconnected,
                NetworkEvent::Disconnected(
                    error.map_or_else(|| "Connection closed".into(), |error| error.to_string()),
                ),
            )
        });
    let inbox = Arc::downgrade(&network.inbox);
    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_futures::spawn_local(async move {
        send(
            &inbox,
            NetworkEvent::Built(builder.build().await.map_err(|error| error.to_string())),
        );
    });
    #[cfg(not(target_arch = "wasm32"))]
    send(
        &inbox,
        NetworkEvent::Built(builder.build().map_err(|error| error.to_string())),
    );
}

fn reconnect(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<ConnectionConfig>,
    mut network: ResMut<Network>,
    mut snapshot: ResMut<ConnectionSnapshot>,
) {
    if keys.just_pressed(KeyCode::KeyR) {
        begin_connection(&mut network, &config, &mut snapshot);
    }
}

fn apply_event(snapshot: &mut ConnectionSnapshot, event: NetworkEvent) {
    match event {
        NetworkEvent::Ready => {
            snapshot.phase = "connected";
            snapshot.detail = "Live server presence (one row per connection).".into();
        }
        NetworkEvent::Failed(error) | NetworkEvent::Built(Err(error)) => {
            snapshot.phase = "error";
            snapshot.detail = error;
        }
        NetworkEvent::Disconnected(detail) => {
            snapshot.phase = "disconnected";
            snapshot.detail = detail;
        }
        NetworkEvent::Built(Ok(_)) => unreachable!("pump owns successful builds"),
    }
    if snapshot.phase != "connected" {
        snapshot.sessions.clear();
        snapshot.local_session = None;
    }
}

fn pump(mut network: ResMut<Network>, mut snapshot: ResMut<ConnectionSnapshot>) {
    if let Some(connection) = &network.connection
        && let Err(error) = connection.frame_tick()
    {
        send(
            &Arc::downgrade(&network.inbox),
            NetworkEvent::Failed(error.to_string()),
        );
    }
    let events = std::mem::take(&mut *network.inbox.lock().expect("network inbox poisoned"));
    let mut next = snapshot.clone();
    for event in events {
        if let NetworkEvent::Built(Ok(connection)) = event {
            network.connection = Some(connection);
        } else {
            apply_event(&mut next, event);
        }
    }
    if next.phase == "connected"
        && let Some(connection) = &network.connection
    {
        next.sessions = connection
            .db
            .presence()
            .iter()
            .map(|row| row.connection_id)
            .collect();
        next.sessions.sort_unstable();
        next.local_session = Some(connection.connection_id());
    }
    if *snapshot != next {
        *snapshot = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_endpoint_before_sdk_builder_can_panic() {
        for good in [
            "ws://127.0.0.1:3001",
            "wss://example.com",
            "https://example.com",
        ] {
            assert!(validate_endpoint(good).is_ok());
        }
        for bad in ["bad URL", "/relative", "ftp://example.com"] {
            assert!(validate_endpoint(bad).is_err());
        }
    }

    #[test]
    fn abandoned_callbacks_cannot_reach_new_attempt() {
        let mut network = Network::default();
        let old_sink = Arc::downgrade(&network.inbox);
        network.inbox = Inbox::default();
        send(&old_sink, NetworkEvent::Ready);
        assert!(old_sink.upgrade().is_none());
        assert!(network.inbox.lock().unwrap().is_empty());
    }

    #[test]
    fn disconnect_and_failure_clear_stale_presence() {
        for event in [
            NetworkEvent::Disconnected("closed".into()),
            NetworkEvent::Failed("offline".into()),
        ] {
            let mut snapshot = ConnectionSnapshot::default();
            apply_event(&mut snapshot, NetworkEvent::Ready);
            snapshot.sessions.push(ConnectionId::from_u128(1));
            snapshot.local_session = snapshot.sessions.first().copied();
            apply_event(&mut snapshot, event);
            assert!(snapshot.sessions.is_empty());
            assert!(snapshot.local_session.is_none());
            assert!(snapshot.summary().contains("unavailable"));
        }
    }

    #[test]
    fn summary_lists_full_connection_ids_and_marks_local_session() {
        let id = ConnectionId::from_u128(1);
        let snapshot = ConnectionSnapshot {
            phase: "connected",
            sessions: vec![id],
            local_session: Some(id),
            ..default()
        };
        assert!(snapshot.summary().contains("Connected clients: 1"));
        assert!(snapshot.summary().contains(&format!("{id} (you)")));
        assert!(
            ConnectionSnapshot::default()
                .summary()
                .contains("unavailable")
        );
    }
}
