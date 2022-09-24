use serde::Serialize;

mod error;
mod scenarios;

#[derive(Serialize)]
pub struct SignupRequest {
    name: String,
    password: String,
}

#[tokio::main]
async fn main() {}
