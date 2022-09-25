use std::thread;
use std::time::Duration;

use error::TestOutcome;
use futures::channel::mpsc::channel;
use futures::stream::StreamExt;
use futures::SinkExt;

use scenarios::read_load::{run, Reader};
use serde::Serialize;
use tokio::runtime::Handle;

mod error;
mod scenarios;

#[derive(Serialize)]
pub struct SignupRequest {
    name: String,
    password: String,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = channel(1);

    let client = Reader::new().await.unwrap();

    let rps = 1000;

    let rt = Handle::current();
    let handle = thread::spawn(move || loop {
        let mut t = tx.clone();
        let client = client.clone();

        rt.spawn(async move {
            let result = run(&client).await.unwrap();
            let elapsed = match result {
                TestOutcome::Ok(elapsed) => elapsed,
                TestOutcome::SlowDown(elapsed) => elapsed,
            };
            t.send(elapsed).await.unwrap();
        });
        thread::sleep(Duration::from_micros(1_000_000 / rps));
    });

    let mut i = 0;
    loop {
        if let Some(duration) = rx.next().await {
            println!("{}: {}", i, duration.num_milliseconds());
            i += 1;
        } else {
            break;
        }
    }

    handle.join().unwrap();
    //let num_clients = 50;

    //let mut clients = Vec::new();
    //for _ in 0..num_clients {
    //    clients.push(tokio::spawn(Reader::new()))
    //}

    //let clients = join_all(clients)
    //    .await
    //    .iter()
    //    .map(|result| result.unwrap().unwrap())
    //    .collect::<Vec<Reader>>();
}
