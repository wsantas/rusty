extern crate hyper;
extern crate hyper_rustls;
extern crate yup_oauth2 as oauth2;
extern crate google_gmail1 as gmail1;
use gmail1::{Error, Scope};
use std::default::Default;
use oauth2::{Authenticator, DefaultAuthenticatorDelegate, ApplicationSecret, MemoryStorage, FlowType};
use gmail1::Gmail;
use std::env;

fn main() {
    println!("Starting rusty");

    let key = env::var("google_secret").unwrap();
    let client_id = env::var("google_client_id").unwrap();
    let mut secret: ApplicationSecret = Default::default();
    secret.client_secret = key;
    secret.client_id = client_id;

    let auth = Authenticator::new(&secret, DefaultAuthenticatorDelegate,
                                  hyper::Client::with_connector(hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new())),
                                  <MemoryStorage as Default>::default(), None);
    let hub = Gmail::new(hyper::Client::with_connector(hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new())), auth);

    println!("Starting rusty3");
    let result = hub.users()
        .messages_get("wsantas@gmail.com", "<398dc2a3d54f4413817308ca57778a34@prdphjobs01>").add_scope(Scope::Readonly)
        .doit();
    println!("Starting rusty4");
    match result {
        Err(e) => match e {
            // The Error enum provides details about what exactly happened.
            // You can also just use its `Debug`, `Display` or `Error` traits
            Error::HttpError(_)
            | Error::MissingAPIKey
            | Error::MissingToken(_)
            | Error::Cancelled
            | Error::UploadSizeLimitExceeded(_, _)
            | Error::Failure(_)
            | Error::BadRequest(_)
            | Error::FieldClash(_)
            | Error::JsonDecodeError(_, _) => println!("{}", e),
        },
        Ok(res) => println!("Success: {:?}", res)

    }
}