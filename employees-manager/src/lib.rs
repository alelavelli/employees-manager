use mongodb::bson::oid::ObjectId;

mod auth;
mod dtos;
mod enums;
mod error;
mod facade;
pub mod middleware;
mod model;
pub mod router;
pub mod service;

type DocumentId = ObjectId;
