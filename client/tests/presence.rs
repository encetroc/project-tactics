//! Real SDK/server lifecycle contract, deliberately separate from offline unit tests.
#![cfg(not(target_arch = "wasm32"))]
use project_tactics_client::module_bindings::{DbConnection, PresenceTableAccess};
use spacetimedb_sdk::{DbContext, Table};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

struct Client(DbConnection);
impl Drop for Client {
    fn drop(&mut self) {
        let _ = self.0.disconnect();
    }
}

fn connect(token: Option<String>) -> (Client, Arc<AtomicBool>, Arc<Mutex<Option<String>>>) {
    let ready = Arc::new(AtomicBool::new(false));
    let applied = ready.clone();
    let saved_token = Arc::new(Mutex::new(None));
    let connected_token = saved_token.clone();
    let connection = DbConnection::builder()
        .with_uri(std::env::var("SPACETIME_TEST_SERVER").expect("use scripts/test-integration.sh"))
        .with_database_name(std::env::var("SPACETIME_TEST_DATABASE").expect("test database"))
        .with_token(token)
        .on_connect(move |connection, _, token| {
            *connected_token.lock().unwrap() = Some(token.to_owned());
            connection
                .subscription_builder()
                .on_applied(move |_| {
                    applied.store(true, Ordering::SeqCst);
                })
                .on_error(|_, error| panic!("subscription: {error}"))
                .subscribe(["SELECT * FROM presence"]);
        })
        .on_connect_error(|_, error| panic!("connect: {error}"))
        .build()
        .expect("build connection");
    (Client(connection), ready, saved_token)
}

fn until(clients: &[&Client], condition: impl Fn() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        for client in clients {
            client.0.frame_tick().expect("frame tick");
        }
        if condition() {
            return;
        }
        assert!(Instant::now() < deadline, "presence condition timed out");
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
#[ignore = "requires published local SpacetimeDB; run scripts/test-integration.sh"]
fn sessions_with_shared_identity_replicate_and_disconnect_independently() {
    let (first, ready, token) = connect(None);
    until(&[&first], || ready.load(Ordering::SeqCst));
    let first_id = first.0.connection_id();
    until(&[&first], || {
        first
            .0
            .db
            .presence()
            .iter()
            .any(|p| p.connection_id == first_id)
    });

    let (second, ready, _) = connect(token.lock().unwrap().clone());
    until(&[&first, &second], || ready.load(Ordering::SeqCst));
    let second_id = second.0.connection_id();
    assert_ne!(first_id, second_id);
    assert_eq!(first.0.identity(), second.0.identity());

    until(&[&first, &second], || {
        [&first, &second].iter().all(|client| {
            let rows: Vec<_> = client
                .0
                .db
                .presence()
                .iter()
                .map(|p| p.connection_id)
                .collect();
            rows.contains(&first_id) && rows.contains(&second_id)
        })
    });
    drop(second);
    until(&[&first], || {
        let rows: Vec<_> = first
            .0
            .db
            .presence()
            .iter()
            .map(|p| p.connection_id)
            .collect();
        rows.contains(&first_id) && !rows.contains(&second_id)
    });

    let (third, ready, _) = connect(token.lock().unwrap().clone());
    until(&[&first, &third], || ready.load(Ordering::SeqCst));
    assert_ne!(third.0.connection_id(), second_id);
    until(&[&first, &third], || {
        first
            .0
            .db
            .presence()
            .iter()
            .any(|p| p.connection_id == third.0.connection_id())
    });
}
