use reqwest::Client;
use serde::Deserialize;
use tokio::sync::OnceCell;

pub mod get_collection_details;
pub mod get_published_file_details;

struct Reqwest;
static CLIENT: OnceCell<Client> = OnceCell::const_new();

impl Reqwest {
    async fn client() -> &'static Client {
        CLIENT.get_or_init(|| async { Client::new() }).await
    }
}

#[derive(Debug, Deserialize)]
pub struct Wrapper<T> {
    response: T,
}
