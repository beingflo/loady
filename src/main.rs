use std::sync::Arc;

use error::TestOutcome;
use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::stream::StreamExt;
use futures::SinkExt;

use scenarios::read_load::{run, Reader};
use serde::Serialize;
use tokio::sync::Semaphore;

mod error;
mod scenarios;

#[derive(Serialize)]
pub struct SignupRequest {
    name: String,
    password: String,
}

async fn create_load(client: Reader, semaphore: Arc<Semaphore>, tx: UnboundedSender<TestOutcome>) {
    loop {
        let mut t = tx.clone();
        let ticket = semaphore.clone().acquire_owned().await.unwrap();
        let client = client.clone();

        tokio::spawn(async move {
            let result = run(&client).await.unwrap();
            t.send(result).await.unwrap();
            drop(ticket);
        });
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = unbounded();

    let client = Reader::new().await.unwrap();
    let max_in_flight = 100;
    let semaphore = Arc::new(Semaphore::new(max_in_flight));

    tokio::spawn(create_load(client, semaphore.clone(), tx));

    loop {
        for msg in rx.next().await {
            if let TestOutcome::Ok(duration) = msg {
                println!("{}", duration.num_milliseconds());
            }
        }
    }
}
