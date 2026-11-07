pub mod cost_store;
pub mod embeddings;
pub mod integration_store;
pub mod pricing;
pub mod session_meta;
pub mod store;
pub mod viking;
pub mod viking_store;

pub use cost_store::CostStore;
pub use integration_store::{IntegrationStore, IntegrationToken};
pub use pricing::estimate_cost_cents;
pub use session_meta::SessionMetaStore;
pub use store::SqliteStore;
pub use viking::VikingClient;
pub use viking_store::VikingStore;

pub fn apply_lean_pragmas(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA cache_size = -2000;
         PRAGMA mmap_size = 0;
         PRAGMA temp_store = MEMORY;",
    )
}
