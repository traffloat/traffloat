use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(long, default_value = "0.0.0.0")]
    pub ip: String,
    #[structopt(long, default_value = common::DEFAULT_PORT_STR)]
    pub port: u16,
}
