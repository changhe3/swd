mod net;
mod prelude;
mod util;

use crate::prelude::*;
use net::{
    get_collection_details::{self, Detail as CollectionDetail},
    get_published_file_details::{self, Detail as FileDetail},
};
use std::{collections::HashMap, fmt::Debug};
use structopt::StructOpt;
use tokio::{
    io::{stdout, AsyncWriteExt},
    main,
};

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
    details: FileDetail,
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
    all_files: HashMap<FileId, WFile>,
}

impl WFiles {
    async fn new(mut params: Params) -> Result<Self> {
        let c_details = get_collection_details::call(params.files.iter().copied()).await?;
        let ids = c_details.details.iter().flat_map(|detail| {
            let inner = detail
                .children
                .as_ref()
                .map(|v| {
                    v.iter().filter_map(|c| {
                        if c.filetype == 0 {
                            Some(c.file_id)
                        } else {
                            None
                        }
                    })
                })
                .into_iter()
                .flatten();
            std::iter::once(detail.file_id).chain(inner)
        });

        todo!()
    }

    fn invalid_id(id: FileId) {
        println!("Invalid File ID: {}", id);
    }
}

#[main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    let params = Params::from_args();
    println!("Hello, world!");
    Ok(())
}
