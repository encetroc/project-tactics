//! The server owns presence. Clients can subscribe, but cannot edit it.
use spacetimedb::{ConnectionId, Identity, ReducerContext, Table};

/// A connection is a session, not an account: two tabs may share an identity.
#[spacetimedb::table(accessor = presence, public)]
pub struct Presence {
    #[primary_key]
    pub connection_id: ConnectionId,
    pub identity: Identity,
}

#[spacetimedb::reducer(client_connected)]
pub fn connected(ctx: &ReducerContext) {
    if let Some(connection_id) = ctx.connection_id() {
        ctx.db.presence().insert(Presence {
            connection_id,
            identity: ctx.sender(),
        });
    }
}

#[spacetimedb::reducer(client_disconnected)]
pub fn disconnected(ctx: &ReducerContext) {
    if let Some(connection_id) = ctx.connection_id() {
        ctx.db.presence().connection_id().delete(connection_id);
    }
}
