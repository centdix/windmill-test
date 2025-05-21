use sqlx::Postgres;

use crate::{error::Result, DB};

#[cfg(feature = "enterprise")]
mod inner {
    pub use crate::stats_ee::*;
}

#[cfg(not(feature = "enterprise"))]
mod inner {
    pub use crate::stats_oss::*;
}

pub use inner::*;