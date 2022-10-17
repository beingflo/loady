use std::sync::Arc;

use chrono::{Duration, Utc};
use error::TestOutcome;
use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::stream::StreamExt;
use futures::SinkExt;

use scenarios::read_load::{run, Reader};
use serde::Serialize;
use statistics::Aggregator;
use tokio::sync::Semaphore;

mod error;
mod scenarios;
mod statistics;

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
            let result = run(&client).await;
            match result {
                Result::Err(error) => println!("{error}"),
                Result::Ok(res) => {
                    t.send(res).await.unwrap();
                    drop(ticket);
                }
            }
        });
    }
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = unbounded();

    let client = Reader::new().await.unwrap();
    let mut max_in_flight = 1;
    let semaphore = Arc::new(Semaphore::new(max_in_flight));

    tokio::spawn(create_load(client, semaphore.clone(), tx));

    let mut last_adjustment = Utc::now();

    let mut num_slow_down = 0;

    let mut aggregator = Aggregator::new();

    loop {
        let msg = rx.next().await.unwrap();

        aggregator.add_request(msg);
        aggregator.recompute_statistics();

        println!("{aggregator}");

        let now = Utc::now();

        if now - last_adjustment > Duration::seconds(1) {
            last_adjustment = now;

            if num_slow_down > 10 {
                // Half the number of requests in flight
                num_slow_down = 0;
                let remove = max_in_flight / 2;
                println!("removing {remove}");
                max_in_flight = max_in_flight - remove;
                let permits = semaphore
                    .clone()
                    .acquire_many_owned(remove as u32)
                    .await
                    .unwrap();
                permits.forget();
            } else {
                // Add 1 to the number of requests in flight
                semaphore.clone().add_permits(1);
                max_in_flight += 1;
            }

            println!("Number of requests in flight: {max_in_flight}");
        }

        if let TestOutcome::SlowDown(_) = msg {
            num_slow_down += 1;
        }
    }
}
