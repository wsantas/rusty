extern crate rusoto_comprehend;
extern crate rusoto_core;
extern crate tokio_core;
extern crate tokio;
use self::rusoto_comprehend::{DetectSentimentResponse, ComprehendClient, Comprehend, DetectSentimentRequest};
use rusoto_core::Region;
use self::tokio::runtime::Runtime;
use rusoto_comprehend::{DetectKeyPhrasesResponse, DetectKeyPhrasesRequest};

#[derive(Deserialize)]
pub struct EmailSentimentForm {
    pub message_id: String,
    pub mailbox: String
}

pub fn check_sentiment(message: String) -> DetectSentimentResponse {
    let mut runtime = Runtime::new().unwrap();
    let client = ComprehendClient::new(Region::UsEast1);
    let detect_sentiment_request = DetectSentimentRequest {
        text: message,
        language_code: "en".parse().unwrap()
    };
    let future = client.detect_sentiment(detect_sentiment_request);
    runtime.block_on(future).unwrap()
}

pub fn detect_key_phrases(message: String) -> DetectKeyPhrasesResponse {
    let mut runtime = Runtime::new().unwrap();
    let client = ComprehendClient::new(Region::UsEast1);
    let detect_key_phrases_request = DetectKeyPhrasesRequest {
        text: message,
        language_code: "en".parse().unwrap()
    };
    let future = client.detect_key_phrases(detect_key_phrases_request);
    runtime.block_on(future).unwrap()
}