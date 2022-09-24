use chrono::Utc;
use reqwest::{self, Client, ClientBuilder};

use crate::{error::TestError, error::TestOutcome, SignupRequest};

pub struct Reader {
    client: Client,
}

impl Reader {
    async fn new() -> Result<Self, TestError> {
        let client = ClientBuilder::new().cookie_store(true).build().unwrap();
        let response = client
            .post("http://localhost:3030/session")
            .json(&SignupRequest {
                name: "test".into(),
                password: "test".into(),
            })
            .send()
            .await?;

        assert_eq!(response.status(), 200);

        Ok(Reader { client })
    }

    async fn teardown(&self) -> Result<(), TestError> {
        let response = self
            .client
            .delete("http://localhost:3030/session")
            .send()
            .await?;

        assert_eq!(response.status(), 200);

        Ok(())
    }
}

pub async fn run(client: &Reader) -> Result<TestOutcome, TestError> {
    let start = Utc::now();
    let response = client
        .client
        .get("http://localhost:3030/notes")
        .send()
        .await?;
    let elapsed = Utc::now() - start;
    assert_eq!(response.status(), 200);

    Ok(TestOutcome::Ok(elapsed))
}