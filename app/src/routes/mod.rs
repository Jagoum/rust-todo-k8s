//! # HTTP Route Handlers
//!
//! Contains all HTTP route handlers organized by functionality:
//! - `public`: Unauthenticated routes (registration, login)
//! - `todos`: Authenticated routes for todo management

pub mod public;
pub mod todos;