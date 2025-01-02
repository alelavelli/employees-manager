use serde::Serialize;

use crate::DocumentId;

#[derive(Serialize)]
pub struct User {
    pub id: DocumentId,
    pub username: String,
}
