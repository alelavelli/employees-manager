//! Service module that contain application services
//!
//! Application services contain core business logic of out Application,
//! they are used by Facades to perform operations, read and store entities on
//! database.
//!
//! Services split into two main categories:
//!     - support services are used by the whole application (other services included)
//!       and provide utilities. They are environment and database service;
//!     - application services are part of the actual application and may use support services.
//!

pub mod access_control;
pub mod admin;
//pub mod company;
pub mod corporate_group;
pub mod db;
pub mod environment;
//pub mod notification;
//pub mod object_storage;
//pub mod project;
//pub mod timesheet;
pub mod user;
