use mongodb::bson::oid::ObjectId;

mod auth;
mod dtos;
mod enums;
mod error;
mod facade;
pub mod middleware;
pub mod model;
pub mod router;
pub mod service;

type DocumentId = ObjectId;
