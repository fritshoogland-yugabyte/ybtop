# ybtop  
Bringing the functionality of the linux 'top' utility to YugabyteDB.  

Please mind that alike the top utility, the session with the longest running time will be shown as first, and the others following in query runtime time, independent whether this query is run via the YSQL or YCQL endpoint, and on which host the query was running:
```
API  server               client               key/db     status       time_s query
YSQL 192.168.66.80        127.0.0.1:50736      yugabyte   active       26.853 select pg_sleep(120);
YCQL 192.168.66.80        127.0.0.1:35518      cr         QUERY         0.235 select avg(permit), avg(permit_recheck), avg( handgun), avg( long_gun), avg( other), avg( multiple), avg( admin), avg( prepawn_handgun), avg( prepawn_long_gun), avg( prepawn_other), avg( redemption_handgun), avg( redemption_long_gun), avg( redemption_other), avg( returned_handgun), avg( returned_long_gun), avg( returned_other), avg( rentals_handgun), avg( rentals_long_gun), avg( private_sale_handgun), avg( private_sale_long_gun), avg( private_sale_other), avg( return_to_seller_handgun), avg( return_to_seller_long_gun), avg( return_to_seller_other), avg( totals) from fa_bg_checks;
```

# Commandline switches
```
USAGE:
    ybtop [OPTIONS]

FLAGS:
        --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -h, --hosts <hosts>          hostnames, comma separated [default: 192.168.66.80,192.168.66.81,192.168.66.82]
    -i, --interval <interval>    refresh interval [default: 3]
    -p, --ports <ports>          ports numbers, comma separated. YSQL:13000, YCQL:12000 [default: 13000,12000]
```

# How to install
This repository contains the sourcecode for ybtop, which means that you need to compile it as executable yourself. This utility is written in [rust](https://www.rust-lang.org). Compiling the utility yourself is easy, and requires no knowledge of rust. Follow these steps:
1. Install the rust langauge suite; goto `https://www.rust-lang.org/tools/install`, and run the installation. (I found that on certain EL linux versions I needed to install the `gcc` and `openssl-devel` packages)
2. Clone this repository `git clone https://github.com/fritshoogland-yugabyte/ybtop.git`.
3. Build the ybtop executable:
```
cd ybtop
cargo build --release
```
4. Run the executable: `./target/release/ybtop`.

Warning: alpha version, built on OSX 12.3, tested on OSX. Testing and feedback welcome!
