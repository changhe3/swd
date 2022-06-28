use std::cell::Cell;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::OnceCell;

use crate::prelude::FileId;

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

struct IterAdapter<I> {
    iter: I,
    len: Cell<usize>,
}

impl<I: Iterator<Item = FileId> + Clone> Serialize for IterAdapter<I> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_seq(
            self.iter
                .clone()
                .inspect(|id| self.len.set(self.len.get() + 1)),
        )
    }
}
