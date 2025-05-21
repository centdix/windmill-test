use crate::server::Smtp;

#[cfg(feature = "enterprise")]
mod inner {
    pub use crate::email_ee::*;
}

#[cfg(not(feature = "enterprise"))]
mod inner {
    pub use crate::email_oss::*;
}

pub use inner::*;