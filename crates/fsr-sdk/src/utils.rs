// SPDX-License-Identifier: MPL-2.0

//! Opt-in project-defined utilities using public core inputs and results.
//!
//! Enable the off-by-default `utils` feature. It adds no dependencies and enables
//! neither `dx12` nor `movable`. Core defaults and native safety contracts remain
//! unchanged; callers apply utility results through the ordinary core API.

pub mod provider;
