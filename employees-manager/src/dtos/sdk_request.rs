use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub name: String,
    pub surname: String,
    pub email: String,
}
