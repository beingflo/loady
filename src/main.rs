use std::{sync::Arc, time::Duration};

use error::TestOutcome;
use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::stream::StreamExt;
use futures::SinkExt;

use scenarios::read_load::{run, Reader};
use serde::Serialize;
use tokio::{sync::Semaphore, time::sleep};

mod error;
mod scenarios;

#[derive(Serialize)]
pub struct SignupRequest {
    name: String,
    password: String,
}

async fn create_load(
    client: Reader,
    semaphore: Arc<Semaphore>,
    tx: UnboundedSender<TestOutcome>,
    rps: u64,
) {
    loop {
        let mut t = tx.clone();
        let ticket = semaphore.clone().acquire_owned().await.unwrap();
        let client = client.clone();

        tokio::spawn(async move {
            let result = run(&client).await.unwrap();
            t.send(result).await.unwrap();
            drop(ticket);
        });
        sleep(Duration::from_micros(1_000_000 / rps)).await;
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = unbounded();

    let client = Reader::new().await.unwrap();
    let max_in_flight = 1;
    let semaphore = Arc::new(Semaphore::new(max_in_flight));

    let rps = 100;

    tokio::spawn(create_load(client, semaphore.clone(), tx, rps));

    loop {
        for msg in rx.next().await {
            println!("{:?}", msg);
        }
    }
}
