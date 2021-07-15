use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Opts {
    #[structopt(long, parse(from_os_str), default_value = "output")]
    pub root_dir: PathBuf,
    #[structopt(long, parse(from_os_str), default_value = "../client")]
    pub client_dir: PathBuf,
    #[structopt(long)]
    pub site_url: Option<String>,
}
