mod cluster;
mod database;
mod image_catalog;
mod object_store;
mod scheduled_backups;

pub use cluster::*;
pub use database::*;
pub use image_catalog::*;
pub use object_store::*;
pub use scheduled_backups::*;

impl Default for ClusterBackupBarmanObjectStoreDataCompression {
    fn default() -> Self {
        ClusterBackupBarmanObjectStoreDataCompression::Gzip
    }
}

impl Default for ClusterBootstrapInitdbImportType {
    fn default() -> Self {
        ClusterBootstrapInitdbImportType::Microservice
    }
}

impl Default for ClusterManagedServicesAdditionalSelectorType {
    fn default() -> Self {
        ClusterManagedServicesAdditionalSelectorType::R
    }
}

impl Default for ClusterPostgresqlSynchronousMethod {
    fn default() -> Self {
        ClusterPostgresqlSynchronousMethod::Any
    }
}
