use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {
    /// hostnames, comma separated.
    #[structopt(short, long, default_value = "192.168.66.80,192.168.66.81,192.168.66.82")]
    hosts: String,
    /// ports numbers, comma separated. YSQL:13000, YCQL:12000
    #[structopt(short, long, default_value = "13000,12000")]
    ports: String,
    /// update interval
    #[structopt(short, long, default_value = "3")]
    update: u64,
    /// show idle sessions
    #[structopt(short, long)]
    idle: bool,
}

fn main() {

    let options = Opts::from_args();
    let hostname_vec: Vec<&str> = options.hosts.split(",").collect();
    let port_vec: Vec<&str> = options.ports.split(",").collect();
    let update_interval: u64 = options.update;
    let idle: bool = options.idle;

    ybtop::display_clients( hostname_vec, port_vec, update_interval, idle );

}
