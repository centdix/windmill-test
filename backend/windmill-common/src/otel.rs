/*
 * Author: Ruben Fiszel
 * Copyright: Windmill Labs, Inc 2022
 * This file and its contents are licensed under the AGPLv3 License.
 * Please see the included NOTICE for copyright information and
 * LICENSE-AGPL for a copy of the license.
 */

#[cfg(feature = "enterprise")]
mod inner {
    pub use crate::otel_ee::*;
}

#[cfg(not(feature = "enterprise"))]
mod inner {
    pub use crate::otel_oss::*;
}

pub use inner::*;