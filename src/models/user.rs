use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)] 
pub struct UserCredentials {
    email: String,
    password: String,
}


