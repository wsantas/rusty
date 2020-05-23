extern crate rusoto_comprehend;
extern crate rusoto_core;
extern crate tokio_core;
extern crate tokio;
use self::rusoto_comprehend::{DetectSentimentResponse, ComprehendClient, Comprehend, DetectSentimentRequest};
use rusoto_core::Region;
use self::tokio_core::reactor::Core;
use self::tokio::runtime::Runtime;
use std::ops::Deref;

pub fn check_sentiment(message: String) -> DetectSentimentResponse {
    let mut runtime = Runtime::new().unwrap();
    let client = ComprehendClient::new(Region::UsEast1);
    let mut core = Core::new().unwrap();
    let detect_sentiment_request = DetectSentimentRequest {
        text: message,
        language_code: "en".parse().unwrap()
    };
    let mut future = client.detect_sentiment(detect_sentiment_request);
    runtime.block_on(future).unwrap()
}