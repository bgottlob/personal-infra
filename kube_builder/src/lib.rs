mod deployment;
mod ingress;
mod secret;
mod service;

pub mod prelude {
    pub use crate::deployment::*;
    pub use crate::ingress::*;
    pub use crate::secret::*;
    pub use crate::service::*;
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
