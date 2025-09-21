//! Facade module contains one Facade for each Router
//!
//! A Facade is responsible to mask the actual application logic and services
//! to the Router avoiding close tiding.

pub mod admin;
pub mod corporate_group_admin;
pub mod corporate_group_user;
pub mod guest;
//pub mod sdk;
pub mod user;
//pub mod web_app;
