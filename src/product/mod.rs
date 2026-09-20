//! Product-level state that is independent of the hardware drivers and UI.
//!
//! This module deliberately owns durable user choices. Hardware-specific
//! persistence (NVS) will implement the storage trait in a later change.

pub mod settings;
