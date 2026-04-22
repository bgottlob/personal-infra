pub mod container;
pub mod deployment;
pub mod http_route;
pub mod ingress;
pub mod persistent_volume_claim;
pub mod sealed_secrets;
pub mod secret;
pub mod service;
pub mod tailscale;

pub mod prelude {
    pub use crate::container::*;
    pub use crate::deployment::*;
    pub use crate::http_route::*;
    pub use crate::ingress::*;
    pub use crate::persistent_volume_claim::*;
    pub use crate::sealed_secrets::*;
    pub use crate::secret::*;
    pub use crate::service::*;
    pub use crate::tailscale;
    pub use crate::PortProtocol;
}

use std::fmt;

pub enum PortProtocol {
    TCP,
    UDP,
}

impl fmt::Display for PortProtocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::TCP => write!(f, "TCP"),
            Self::UDP => write!(f, "UDP"),
        }
    }
}
