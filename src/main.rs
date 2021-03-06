mod net;
mod prelude;
mod util;

use crate::prelude::*;
use chrono::{DateTime, Utc};
use dialoguer::Input;
use itertools::Itertools;
use net::{
    get_collection_details,
    get_published_file_details::{self, DetailInner},
};
use std::{
    collections::{HashMap, HashSet},
    fmt::{format, Debug},
    fs::File,
    path::PathBuf,
    process::Command,
    str::FromStr,
};
use std::{env::current_dir, io::Write as IoWrite};
use structopt::StructOpt;
use util::PrettyCmd;

///
/// (This software has not been extensively tested, use at your own risk. Require steamcmd under PATH)
///
/// You would need [SteamCMD](https://developer.valvesoftware.com/wiki/SteamCMD#Downloading_SteamCMD) to use this software, and remember to include it under the PATH environment variable.
/// A command-line utility to download workshop item and collections from steam workshop.
/// This software assembles a command for SteamCMD to execute. By default, this command is only printed to standard output, you need to use the `-e` flag to automatically execute the command.
/// The default download directory is /path/to/steamcmd/steamapps/workshop/content/. You can set an alternative location with `-o`.
#[derive(Debug, StructOpt)]
struct Params {
    /// Execute the produced command through steamcmd, otherwise the command is only printed to standard output and need to be executed manually.
    #[structopt(short, long)]
    exec: bool,

    /// Review each mod one by one. Input yes/no/skip for each mod or collection.
    /// The option 'skip', otherwise equivalent to 'no', can be used to skip rest of the mods in the context of a collection.
    #[structopt(short, long)]
    review: bool,

    /// Steam username for non-anonymous download
    #[structopt(short, long, default_value = "anonymous")]
    username: String,

    /// Set the path of the download location. The path will be passed to force_install_dir in SteamCMD.
    #[structopt(short, long, name = "path")]
    output: Option<PathBuf>,

    /// Save the mod orders of collections to specified format to the current working directory.
    #[structopt(
        long,
        takes_value(true),
        require_equals(true),
        possible_values(&["simple", "csv"]),
        name = "format",
    )]
    save: Option<String>,

    /// File IDs of the mods and collections to download, can be found at the end of the url for each workshop item.
    files: Vec<FileId>,
}

#[derive(Debug)]
struct WFile {
    file_id: FileId,
    app_id: FileId,
    children: Option<Vec<FileId>>,
    title: String,
    _description: String,
    time_created: DateTime<Utc>,
    time_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReviewOptions {
    Yes,
    No,
    Skip,
}

impl FromStr for ReviewOptions {
    type Err = color_eyre::eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "y" | "yes" => Ok(Self::Yes),
            "n" | "no" => Ok(Self::No),
            "skip" => Ok(Self::Skip),
            _ => Err(color_eyre::eyre::eyre!("invalid input")),
        }
    }
}

impl ToString for ReviewOptions {
    fn to_string(&self) -> String {
        match self {
            ReviewOptions::Yes => "yes",
            ReviewOptions::No => "no",
            ReviewOptions::Skip => "skip",
        }
        .into()
    }
}

impl WFile {
    fn is_collection(&self) -> bool {
        self.children.is_some()
    }

    fn prompt(&self, review: bool) -> Result<ReviewOptions> {
        let item = if self.is_collection() {
            "collection"
        } else {
            "mod"
        };
        println!(
            "Found {} \t\t {}: \t {} \t\t Created at: \t {} \t\t Updated at: \t {} ",
            item,
            self.file_id,
            self.title,
            self.time_created.naive_local(),
            self.time_updated.naive_local(),
        );
        let res = if review {
            let mut input = Input::<ReviewOptions>::new();
            input.with_prompt("Install? [yes/no/skip]");
            input.default(ReviewOptions::Yes);
            input.interact_text()?
        } else {
            ReviewOptions::Yes
        };
        Ok(res)
    }
}

#[derive(Debug)]
struct WFiles {
    params: Params,
    all_files: HashMap<FileId, WFile>,
}

impl WFiles {
    fn new(mut params: Params) -> Result<Self> {
        // retieve the mod ids of each collection listed in params.files
        let mut c_details = get_collection_details::call(params.files.iter().copied())?;

        // retrieve details about all mods and collections listed in params.files, plus all submods inside each collection
        let all_ids = c_details.details.values().flat_map(|detail| {
            let inner = detail.children.iter().flat_map(|v| {
                // remove linked collections
                v.iter()
                    .filter_map(|c| (c.filetype == 0).then_some(c.file_id))
            });
            std::iter::once(detail.file_id).chain(inner).dedup()
        });
        let f_details = get_published_file_details::call(all_ids)?;

        let all_files = f_details
            .details
            .into_iter()
            .filter_map(|d| {
                if !d.is_valid_item() {
                    Self::invalid_id(d.file_id);
                    return None;
                }

                let DetailInner {
                    app_id,
                    title,
                    description,
                    time_created,
                    time_updated,
                } = d.inner.unwrap(); // d.inner is guaranteed non-null since d.result == 1

                let file_id = d.file_id;
                let children = {
                    let children = c_details
                        .details
                        .get_mut(&file_id)
                        .and_then(|d| d.children.as_mut());
                    children.map(|children| {
                        children.sort_by_key(|c| c.sortorder);
                        children
                            .iter()
                            .filter_map(|c| (c.filetype == 0).then(|| c.file_id))
                            .collect::<Vec<_>>()
                    })
                };

                let wfile = WFile {
                    file_id,
                    app_id,
                    children,
                    title,
                    _description: description,
                    time_created,
                    time_updated,
                };
                Some((file_id, wfile))
            })
            .collect::<HashMap<_, _>>();

        // Remove invalid mod ids
        params.files.retain(|id| all_files.contains_key(id));
        Ok(Self { params, all_files })
    }

    fn invalid_id(id: FileId) {
        println!("Invalid File ID: {}", id);
    }

    fn build_cmd(&self) -> Result<Command> {
        let mut cmd = Command::new("steamcmd");
        if let Some(path) = self.params.output.as_deref() {
            cmd.arg("+force_install_dir");
            cmd.arg(path);
        }

        cmd.arg("+login").arg(&self.params.username);

        let wd = current_dir().unwrap();
        let mut all_mods = HashSet::new();

        for file_id in self.params.files.iter() {
            let file = &self.all_files[file_id];
            if ReviewOptions::Yes != file.prompt(self.params.review)? {
                continue;
            }

            if !file.is_collection() {
                if all_mods.insert(file_id) {
                    cmd.arg(format!(
                        "+workshop_download_item {} {}",
                        file.app_id, file.file_id
                    ));
                }
            } else {
                let save_path = self
                    .params
                    .save
                    .as_ref()
                    .map(|_| wd.join(format!("{} {}.csv", file_id, file.title)));

                let mut save_file = save_path.map(File::create).transpose()?;

                for file_id in file.children.as_ref().unwrap().iter() {
                    let inner_file = &self.all_files[file_id];
                    match inner_file.prompt(self.params.review)? {
                        ReviewOptions::Yes => {
                            if all_mods.insert(&inner_file.file_id) {
                                cmd.arg(format!(
                                    "+workshop_download_item {} {}",
                                    inner_file.app_id, inner_file.file_id
                                ));
                            }

                            if let Some(save_file) = save_file.as_mut() {
                                match self.params.save.as_deref().unwrap() {
                                    "simple" => writeln!(save_file, "{}", file_id)?,
                                    "csv" => writeln!(
                                        save_file,
                                        "{}\t{}\t{}\t{}",
                                        inner_file.file_id,
                                        inner_file.title,
                                        inner_file.time_created,
                                        inner_file.time_updated
                                    )?,
                                    _ => unreachable!(),
                                };
                            }
                        }
                        ReviewOptions::No => {
                            continue;
                        }
                        ReviewOptions::Skip => {
                            break;
                        }
                    }
                }

                if let Some(save_file) = save_file.as_mut() {
                    save_file.flush()?;
                }
            }
        }

        cmd.arg("+quit");
        Ok(cmd)
    }

    fn run(&self) -> Result<()> {
        let mut cmd = self.build_cmd()?;
        if !self.params.exec {
            println!("\n{}", PrettyCmd::new(&cmd))
        } else if let Ok(mut proc) = cmd.spawn() {
            proc.wait().unwrap();
        } else {
            return Err(color_eyre::eyre::eyre!("steamcmd failed"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null() -> Result<()> {
        let params = Params::from_iter(["swd"].into_iter());
        __main__(params)
    }

    #[test]
    fn test_collection() -> Result<()> {
        let params = Params::from_iter(["swd", "368330611"].into_iter());
        __main__(params)
    }

    #[test]
    fn test_output() -> Result<()> {
        let params = Params::from_iter(
            [
                "swd",
                "-o",
                r"C:\User\admin\My Documents\My Mods",
                "368330611",
            ]
            .into_iter(),
        );
        __main__(params)
    }

    // #[test]
    // fn test_collection_review() -> Result<()> {
    //     let params =
    //         Params::from_iter(["swd", "-r", "--save=csv", "368330611", "116676096"].into_iter());
    //     __main__(params)
    // }
}

fn __main__(params: Params) -> Result<()> {
    if params.files.is_empty() {
        Params::clap().print_long_help()?;
    } else {
        let wfiles = WFiles::new(params)?;
        wfiles.run()?;
    }

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    __main__(Params::from_args())
}
