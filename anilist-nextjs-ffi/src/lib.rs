mod nextjs;

pub use nextjs::{NextJsError, deserialize_nextjs};

uniffi::setup_scaffolding!();
