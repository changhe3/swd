mod net;
mod prelude;
mod util;

use crate::prelude::*;
use net::{get_collection_details, get_published_file_details::Detail};
use serde::Deserialize;
use std::collections::HashMap;
use structopt::StructOpt;
use tokio::main;

#[derive(Debug, StructOpt)]
#[structopt(about = "Download workshop item and collections from steam workshop")]
struct Params {
    #[structopt(
        short,
        long,
        help = "Execute the produced command through steamcmd, otherwise the command is only printed to stdout"
    )]
    exec: bool,

    #[structopt(short, long, help = "Review each mod one by one")]
    review: bool,

    #[structopt(short, long, default_value = "anonymous")]
    username: String,

    #[structopt(
        long,
        takes_value(true),
        require_equals(true),
        possible_values(&["simple", "csv"]),
        help = "Save the mod orders of collections to specified format to the current working directory"
    )]
    save: Option<String>,

    #[structopt(help = "File IDs of the mods and collections to download")]
    files: Vec<FileId>,
}

#[derive(Debug)]
struct WFile {
    id: FileId,
    children: Option<Vec<FileId>>,
    details: Option<Detail>,
}

impl WFile {
    fn is_collection(&self) -> bool {
        self.children.is_some()
    }

    async fn retrieve_details(&mut self) {
        todo!()
    }

    async fn save_order() {
        todo!()
    }
}

#[derive(Debug)]
struct WFiles {
    params: Params,
    listed_files: Vec<FileId>,
    all_files: HashMap<FileId, WFile>,
}

impl WFiles {
    async fn new(mut params: Params) -> Result<Self> {
        let listed_files = params.files.split_off(0);
        let response = get_collection_details::call(&listed_files).await?;
        todo!()
    }
}

#[main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let params = Params::from_args();
    println!("Hello, world!");
    Ok(())
}
