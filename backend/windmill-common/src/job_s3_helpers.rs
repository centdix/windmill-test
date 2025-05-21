use std::future::Future;

use crate::{
    error::Error,
    s3_helpers::{ObjectStoreResource, StorageResourceType},
};

#[cfg(feature = "enterprise")]
mod inner {
    pub use crate::job_s3_helpers_ee::*;
}

#[cfg(not(feature = "enterprise"))]
mod inner {
    pub use crate::job_s3_helpers_oss::*;
}

pub use inner::*;