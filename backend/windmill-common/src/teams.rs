#[cfg(feature = "enterprise")]
mod inner {
    pub use crate::teams_ee::*;
}

#[cfg(not(feature = "enterprise"))]
mod inner {
    pub use crate::teams_oss::*;
}

pub use inner::*;