use std::collections::VecDeque;
use std::pin::pin;
use std::time::Duration;

use async_stream::stream;
use futures::{Stream, StreamExt};
use tokio::time::{sleep, Instant};

pub async fn smooth_stream<S>(input: S, max_interval: Duration) -> impl Stream<Item = S::Item>
where
    S: Stream + Unpin + Send + 'static,
    S::Item: Send,
{
    stream! {
        let mut last_output_time = Instant::now();
        let mut last_input_time = Instant::now();
        let mut intervals = VecDeque::with_capacity(10);

        let mut input = pin!(input);
        while let Some(item) = input.next().await {
            let now = Instant::now();
            let input_interval = now.duration_since(last_input_time);
            intervals.push_back(input_interval);
            if intervals.len() > 10 {
                intervals.pop_front();
            }

            let avg_input_interval = intervals.iter().sum::<Duration>() / u32::try_from(intervals.len()).unwrap_or(u32::MAX);
            let interval = avg_input_interval.min(max_interval);

            let time_since_last_output = now.duration_since(last_output_time);
            if time_since_last_output < interval {
                sleep(interval - time_since_last_output).await;
            }

            yield item;
            last_output_time = Instant::now();
            last_input_time = now;
        }
    }
}
