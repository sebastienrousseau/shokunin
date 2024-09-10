pub mod compiler;
pub mod lang;
pub mod languages;
pub mod loggers;
pub mod macros;
pub mod metadata;
pub mod models;
pub mod modules;
pub mod server;
pub mod utilities;

// Re-export commonly used items
pub use compiler::service::compile;
pub use languages::translate;
pub use loggers::init_logger;
pub use server::serve::start;
pub use utilities::uuid::generate_unique_string;
