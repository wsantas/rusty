extern crate rusoto_comprehend;
use rusoto_comprehend::{DetectSentimentResponse, ComprehendClient};

pub fn check_sentiment(message: String) -> DetectSentimentResponse {
    let client = ComprehendClient::new(Region::UsEast1);
}