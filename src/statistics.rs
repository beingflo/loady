use std::fmt::{self, Display, Formatter};

use chrono::{DateTime, Duration, Utc};

use crate::error::TestOutcome;

#[derive(Copy, Clone)]
enum RequestStatus {
    Ok,
    Delayed,
}

#[derive(Copy, Clone)]
pub struct RequestStatistic {
    timestamp: DateTime<Utc>,
    duration: Duration,
    status: RequestStatus,
}
pub struct Aggregator {
    requests: Vec<RequestStatistic>,
    last_second_index: usize,

    average_duration: f64,
    average_duration_last_second: f64,

    num_requests_last_second: u64,
    num_delayed_last_second: u64,
}

impl Aggregator {
    pub fn new() -> Self {
        Aggregator {
            requests: Vec::new(),
            last_second_index: 0,
            average_duration: 0.0,
            average_duration_last_second: 0.0,
            num_requests_last_second: 0,
            num_delayed_last_second: 0,
        }
    }

    pub fn recompute_statistics(&mut self) {
        let now = Utc::now();

        loop {
            if self.last_second_index >= self.requests.len() {
                break;
            }

            let old_request = self.requests[self.last_second_index];
            if old_request.timestamp >= now - Duration::seconds(1) {
                break;
            }

            if let RequestStatus::Delayed = old_request.status {
                self.num_delayed_last_second -= 1;
            }

            self.average_duration_last_second = (self.average_duration_last_second
                - (old_request.duration.num_microseconds().unwrap() as f64
                    / self.num_requests_last_second as f64))
                * (self.num_requests_last_second as f64
                    / (self.num_requests_last_second - 1) as f64);

            self.num_requests_last_second -= 1;
            self.last_second_index += 1;
        }
    }

    pub fn add_request(&mut self, request: TestOutcome) {
        self.num_requests_last_second += 1;

        let duration = match request {
            TestOutcome::Ok(duration) => {
                self.requests.push(RequestStatistic {
                    timestamp: Utc::now(),
                    duration: duration,
                    status: RequestStatus::Ok,
                });
                duration
            }
            TestOutcome::SlowDown(duration) => {
                self.num_delayed_last_second += 1;
                self.requests.push(RequestStatistic {
                    timestamp: Utc::now(),
                    duration: duration,
                    status: RequestStatus::Delayed,
                });
                duration
            }
        };
        let old_length = self.num_requests_last_second as f64;
        let new_len = old_length + 1.0;

        self.average_duration_last_second = self.average_duration_last_second
            * (old_length / new_len)
            + duration.num_microseconds().unwrap() as f64 / new_len;
    }
}

impl Display for Aggregator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "rps: {}, avg latency: {:.2})",
            self.num_requests_last_second,
            self.average_duration_last_second / 1000.0
        )
    }
}
