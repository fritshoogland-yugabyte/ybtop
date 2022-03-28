use crate::AllConnections::{Connections, InboundConnections};
use port_scanner::scan_port_addr;
use serde_derive::{Deserialize, Serialize};
use std::process;
use std::{thread, time};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum AllConnections {
    Connections {
        connections: Vec<Connection>,
    },
    InboundConnections {
        inbound_connections: Vec<InboundConnection>,
    },
    Empty {},
}

#[derive(Serialize, Deserialize, Debug)]
struct Connection {
    process_start_time: String,
    application_name: String,
    backend_type: String,
    backend_status: String,
    db_oid: Option<u32>,
    db_name: Option<String>,
    host: Option<String>,
    port: Option<String>,
    query: Option<String>,
    query_start_time: Option<String>,
    transaction_start_time: Option<String>,
    process_running_for_ms: Option<u32>,
    transaction_running_for_ms: Option<u32>,
    query_running_for_ms: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct InboundConnection {
    remote_ip: String,
    state: String,
    processed_call_count: u32,
    connection_details: Option<ConnectionDetails>,
    calls_in_flight: Option<Vec<CallsInFlight>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ConnectionDetails {
    cql_connection_details: CqlConnectionDetails,
}

#[derive(Serialize, Deserialize, Debug)]
struct CqlConnectionDetails {
    keyspace: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct CallsInFlight {
    elapsed_millis: u32,
    cql_details: CqlDetails,
}

#[derive(Serialize, Deserialize, Debug)]
struct CqlDetails {
    #[serde(rename = "type")]
    call_type: String,
    call_details: Vec<CallDetails>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct CallDetails {
    sql_id: Option<String>,
    sql_string: String,
    params: Option<String>,
}

/*
pub struct YsqlPresentation {
    server: String,
    client: String,
    db_name: String,
    application_name: String,
    query_running_for_ms: u32,
    client_status: String,
    query: String,
}

pub struct YcqlPresentation {
    server: String,
    client: String,
    keyspace_name: String,
    processed_call_count: u32,
    elapsed_ms: u32,
    call_type: String,
    sql_string: String,
}

 */

struct GeneralPresentation {
    api: String,
    server: String,
    client: String,
    keyspace_db_name: String,
    status: String,
    query_time_ms: u32,
    query: String,
}

pub fn display_clients(hostname_vec: Vec<&str>, port_vec: Vec<&str>, refresh_interval: u64) {
    let time_to_sleep = time::Duration::from_secs(refresh_interval);
    loop {
        read_rpcz_http(&hostname_vec, &port_vec);
        thread::sleep(time_to_sleep);
    }
}

fn scan_and_parse(hostname: &str, port: &str) -> AllConnections {
    if scan_port_addr(format!("{}:{}", hostname, port)) {
        let get_result = reqwest::blocking::get(format!("http://{}:{}/rpcz", hostname, port))
            .unwrap()
            .text()
            .unwrap();
        parse_result(get_result)
    } else {
        return AllConnections::Empty {};
    }
}

fn parse_result(http_data: String) -> AllConnections {
    serde_json::from_str(&http_data).unwrap_or_else(|e| {
        eprintln!("Error parsing json data: {}", e);
        process::exit(1);
    })
}
fn read_rpcz_http(hostname_vec: &Vec<&str>, port_vec: &Vec<&str>) {
    //let mut ysqlactivity: Vec<YsqlPresentation> = Vec::new();
    //let mut ycqlactivity: Vec<YcqlPresentation> = Vec::new();
    let mut database_activity: Vec<GeneralPresentation> = Vec::new();
    for hostname in hostname_vec {
        for port in port_vec {
            /*
            if scan_port_addr( format!("{}:{}", hostname, port)) {
                let parse_result: AllConnections = serde_json::from_str(&get_result).unwrap_or_else(|e| {
                    println!("Error parsing json data: {}", e);
                    process::exit(1);
                });
             */
            let parse_result = scan_and_parse(&hostname, &port);
            //dbg!(&parse_result);
            match parse_result {
                Connections { connections } => {
                    for connection in connections {
                        if connection.backend_status != "" && connection.backend_status != "idle" {
                            /*
                            ysqlactivity.push( YsqlPresentation {
                                server: hostname.to_string(),
                                client: format!("{}:{}", connection.host.unwrap_or_default(), connection.port.unwrap_or_default()),
                                db_name: connection.db_name.unwrap_or_default(),
                                application_name: connection.application_name.to_string(),
                                query_running_for_ms: connection.query_running_for_ms.unwrap_or_default(),
                                client_status: connection.backend_status.to_string(),
                                query: connection.query.unwrap_or_default()
                            });
                             */
                            database_activity.push(GeneralPresentation {
                                api: String::from("YSQL"),
                                server: hostname.to_string(),
                                client: format!(
                                    "{}:{}",
                                    connection.host.unwrap_or_default(),
                                    connection.port.unwrap_or_default()
                                ),
                                keyspace_db_name: connection.db_name.unwrap_or_default(),
                                status: connection.backend_status.to_string(),
                                query_time_ms: connection.query_running_for_ms.unwrap_or_default(),
                                query: connection.query.unwrap_or_default(),
                            });
                        }
                    }
                }
                InboundConnections {
                    inbound_connections,
                } => {
                    for connection in inbound_connections {
                        if connection.calls_in_flight.is_some() {
                            for cif in connection.calls_in_flight.unwrap() {
                                let keyspace_name = match connection.connection_details.as_ref() {
                                    Some(details) => {
                                        details.cql_connection_details.keyspace.clone()
                                    }
                                    None => String::from(""),
                                };
                                let sql_string = if cif.cql_details.call_details.len() == 1 {
                                    cif.cql_details.call_details[0].sql_string.to_string()
                                } else {
                                    format!(
                                        "Number of statements: {}",
                                        cif.cql_details.call_details.len().to_string()
                                    )
                                };
                                /*
                                ycqlactivity.push( YcqlPresentation {
                                    server: hostname.to_string(),
                                    client: connection.remote_ip.clone(),
                                    keyspace_name: keyspace_name,
                                    processed_call_count: connection.processed_call_count,
                                    elapsed_ms: cif.elapsed_millis,
                                    call_type: cif.cql_details.call_type,
                                    sql_string: sql_string.to_string()
                                });
                                 */
                                database_activity.push(GeneralPresentation {
                                    api: String::from("YCQL"),
                                    server: hostname.to_string(),
                                    client: connection.remote_ip.clone(),
                                    keyspace_db_name: keyspace_name,
                                    status: cif.cql_details.call_type,
                                    query_time_ms: cif.elapsed_millis,
                                    query: sql_string.to_string(),
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    database_activity.sort_by_key(|d| d.query_time_ms);
    database_activity.reverse();
    std::process::Command::new("clear").status().unwrap();
    println!(
        "{:4} {:20} {:20} {:10} {:10} {:>8} {}",
        "API", "server", "client", "key/db", "status", "time_s", "query"
    );
    for row in database_activity {
        println!(
            "{:4} {:20} {:20} {:10} {:10} {:8.3} {}",
            row.api,
            row.server,
            row.client,
            row.keyspace_db_name,
            row.status,
            row.query_time_ms as f32 / 1000.0,
            row.query
        );
    }
    /*
    for row in ysqlactivity {
        println!("YSQL: {} {} {} {} {} {} {}",
        row.server,
        row.client,
        row.db_name,
        row.application_name,
        row.query_running_for_ms,
        row.client_status,
        row.query);
    };
    for row in ycqlactivity {
        println!("YCQL: {} {} {} {} {} {} {}",
        row.server,
        row.client,
        row.keyspace_name,
        row.processed_call_count,
        row.elapsed_ms,
        row.call_type,
        row.sql_string);
    };
     */
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ysql_checkpointer() {
        let http_result = r#"
{
    "connections": [
        {
            "process_start_time": "2022-03-27 10:28:59.884701+00",
            "application_name": "",
            "backend_type": "checkpointer",
            "backend_status": ""
        }
    ]
}
        "#;
        let result = parse_result(http_result.to_string());
        let from_enum = match result {
            Connections { connections } => {
                connections[0].backend_type.clone()
            },
            _ => String::from("")
        };
        assert_eq!(from_enum, "checkpointer");
    }

    #[test]
    fn parse_ysql_idle_process() {
        let http_result = r#"
{
    "connections": [
            {
            "db_oid": 13281,
            "db_name": "yugabyte",
            "query": "select pg_sleep(120);",
            "process_start_time": "2022-03-27 15:55:22.587029+00",
            "query_start_time": "2022-03-28 12:32:39.503558+00",
            "application_name": "ysqlsh",
            "backend_type": "client backend",
            "backend_status": "idle",
            "host": "127.0.0.1",
            "port": "50736"
        }
    ]
}
        "#;
        let result = parse_result(http_result.to_string());
        let from_enum = match result {
            Connections { connections } => {
                connections[0].backend_type.clone()
            },
            _ => String::from("")
        };
        assert_eq!(from_enum, "client backend");
    }

    #[test]
    fn parse_ysql_active_process() {
        let http_result = r#"
{
    "connections": [
        {
            "db_oid": 13281,
            "db_name": "yugabyte",
            "query": "select pg_sleep(120);",
            "process_start_time": "2022-03-27 15:55:22.587029+00",
            "process_running_for_ms": 76583532,
            "transaction_start_time": "2022-03-28 13:11:38.653675+00",
            "transaction_running_for_ms": 7466,
            "query_start_time": "2022-03-28 13:11:38.653675+00",
            "query_running_for_ms": 7466,
            "application_name": "ysqlsh",
            "backend_type": "client backend",
            "backend_status": "active",
            "host": "127.0.0.1",
            "port": "50736"
        }
    ]
}
        "#;
        let result = parse_result(http_result.to_string());
        let from_enum = match result {
            Connections { connections } => {
                connections[0].backend_type.clone()
            },
            _ => String::from("")
        };
        assert_eq!(from_enum, "client backend");
    }

    #[test]
    fn parse_ycql_no_connections() {
        let http_result = r#"
{}
        "#;
        let result = parse_result(http_result.to_string());
        let from_enum = match result {
            _ => String::from("Empty")
        };
        assert_eq!(from_enum, "Empty");
    }

    #[test]
    fn parse_ycql_idle_connections() {
        let http_result = r#"
{
    "inbound_connections": [
        {
            "remote_ip": "127.0.0.1:35518",
            "state": "OPEN",
            "processed_call_count": 2
        },
        {
            "remote_ip": "127.0.0.1:35516",
            "state": "OPEN",
            "processed_call_count": 13
        }
    ]
}
        "#;
        let result = parse_result(http_result.to_string());
        let count_connections = match result {
            InboundConnections { ref inbound_connections} => {
                inbound_connections.len()
            },
            _ => 0
        };
        assert_eq!(count_connections, 2);
        let remote_ip = match result {
            InboundConnections { inbound_connections} => {
                inbound_connections[0].remote_ip.clone()
            },
            _ => String::from("")
        };
        assert_eq!(remote_ip, "127.0.0.1:35518");
    }

    #[test]
    fn parse_ycql_query() {
        let http_result = r#"
{
    "inbound_connections": [
        {
            "remote_ip": "127.0.0.1:35518",
            "state": "OPEN",
            "processed_call_count": 20,
            "connection_details": {
                "cql_connection_details": {
                    "keyspace": "cr"
                }
            },
            "calls_in_flight": [
                {
                    "elapsed_millis": 252,
                    "cql_details": {
                        "type": "QUERY",
                        "call_details": [
                            {
                                "sql_string": "select avg(permit), avg(permit_recheck), avg( handgun), avg( long_gun), avg( other), avg( multiple), avg( admin), avg( prepawn_handgun), avg( prepawn_long_gun), avg( prepawn_other), avg( redemption_handgun), avg( redemption_long_gun), avg( redemption_other), avg( returned_handgun), avg( returned_long_gun), avg( returned_other), avg( rentals_handgun), avg( rentals_long_gun), avg( private_sale_handgun), avg( private_sale_long_gun), avg( private_sale_other), avg( return_to_seller_handgun), avg( return_to_seller_long_gun), avg( return_to_seller_other), avg( totals) from fa_bg_checks;"
                            }
                        ]
                    }
                }
            ]
        }
    ]
}
        "#;
        let result = parse_result(http_result.to_string());
        let remote_ip = match result {
            InboundConnections { ref inbound_connections} => {
                inbound_connections[0].remote_ip.clone()
            },
            _ => String::from("")
        };
        assert_eq!(remote_ip, "127.0.0.1:35518");
        let keyspace = match result {
            InboundConnections { ref inbound_connections} => {
                match &inbound_connections[0].connection_details {
                    Some(details) => {
                        details.cql_connection_details.keyspace.clone()
                    },
                    None => String::from("")
                }
            },
            _ => String::from("")
        };
        assert_eq!(keyspace, "cr");
        let sql_string = match result {
            InboundConnections { ref inbound_connections} => {
                match &inbound_connections[0].calls_in_flight {
                    Some(cif) => {
                        cif[0].cql_details.call_details[0].sql_string.clone()
                    },
                    None => String::from("")
                }
            },
            _ => String::from("")
        };
        assert_eq!(sql_string, "select avg(permit), avg(permit_recheck), avg( handgun), avg( long_gun), avg( other), avg( multiple), avg( admin), avg( prepawn_handgun), avg( prepawn_long_gun), avg( prepawn_other), avg( redemption_handgun), avg( redemption_long_gun), avg( redemption_other), avg( returned_handgun), avg( returned_long_gun), avg( returned_other), avg( rentals_handgun), avg( rentals_long_gun), avg( private_sale_handgun), avg( private_sale_long_gun), avg( private_sale_other), avg( return_to_seller_handgun), avg( return_to_seller_long_gun), avg( return_to_seller_other), avg( totals) from fa_bg_checks;");
    }

    #[test]
    fn parse_ycql_batch() {
        let http_result = r#"
{
    "inbound_connections": [
        {
            "remote_ip": "127.0.0.1:35692",
            "state": "OPEN",
            "processed_call_count": 135,
            "connection_details": {
                "cql_connection_details": {
                    "keyspace": "cr"
                }
            },
            "calls_in_flight": [
                {
                    "elapsed_millis": 6,
                    "cql_details": {
                        "type": "BATCH",
                        "call_details": [
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Alabama, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u001C,, \u0000\u0000\u001C\u001C, n/a, \u0000\u0000\u0001B, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\t, \u0000\u0000\u0000\u000B, n/a, \u0000\u0000\u0005\u000F, \u0000\u0000\u0005, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000D.]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Alaska, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0005ó, \u0000\u0000\u0007l, n/a, \u0000\u0000\u0000L, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0006, \u0000\u0000\u0000\f, n/a, \u0000\u0000\u0000|, \u0000\u0000\u0000£, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000\u000EÜ]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Arizona, \u0000\u0000\nG, n/a, \u0000\u0000\u0015_, \u0000\u0000\u0010Í, n/a, \u0000\u0000\u0000å, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0004, \u0000\u0000\u0000\u0002, n/a, \u0000\u0000\u0002-, \u0000\u0000\u0001Á, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u00005L]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Arkansas, \u0000\u0000\tÖ, n/a, \u0000\u0000\f­, \u0000\u0000\r\", n/a, \u0000\u0000\u0000Î, \u0000\u0000\u0000\u0002, \u0000\u0000\u0000\u0006, \u0000\u0000\u0000\u0013, n/a, \u0000\u0000\u00022, \u0000\u0000\u0005j, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000,*]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, California, \u0000\u0000^ö, n/a, \u0000\u0000F‘, \u0000\u0000:c, n/a, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000ßê]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Colorado, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u001FQ, \u0000\u0000\u001E‘, n/a, \u0000\u0000\fM, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000J/]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Connecticut, \u0000\u0000\u0014ñ, n/a, \u0000\u0000\tú, \u0000\u0000\bK, n/a, \u0000\u0000\u00009, \u0000\u0000\u00023, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000)¢]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Delaware, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0002\u000B, \u0000\u0000\u0001Ü, n/a, \u0000\u0000\u0000\u000E, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000\u0003õ]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, District of Columbia, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0010, n/a, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000\u0000\u0010]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Florida, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000SE, \u0000\u0000*G, n/a, \u0000\u0000\u0002Ù, \u0000\u0000\u0001`, \u0000\u0000\u0000\u0005, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0002\u0004, \u0000\u0000\u0001“, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000…a]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Georgia, \u0000\u0000!D, n/a, \u0000\u0000\u001A), \u0000\u0000\u0012q, n/a, \u0000\u0000\u00010, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0019, \u0000\u0000\u0000\u001F, n/a, \u0000\u0000\u0003Œ, \u0000\u0000\u0005\u001F, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000Wñ]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Guam, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0000\n, \u0000\u0000\u0000\u0019, n/a, \u0000\u0000\u0000\u0001, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000\u0000$]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Hawaii, \u0000\u0000\u0002`, n/a, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000\u0002`]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Idaho, \u0000\u0000\u0005„, n/a, \u0000\u0000\u0006[, \u0000\u0000\u000Bk, n/a, \u0000\u0000\u0000c, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0001, \u0000\u0000\u0000\u0007, n/a, \u0000\u0000\u0000Ç, \u0000\u0000\u0001þ, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000\u001Az]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Illinois, \u0000\u0000d;, n/a, \u0000\u0000\u0014l, \u0000\u0000\u0014l, n/a, \u0000\u0000\u0001\u0007, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000Ž\u001A]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Indiana, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0016E, \u0000\u0000\u0016:, n/a, \u0000\u0000\u0000É, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0001, n/a, \u0000\u0000\u0000\u0001, \u0000\u0000\u0001„, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000.Î]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Iowa, \u0000\u0000\r\r, n/a, \u0000\u0000\u0000\u0011, \u0000\u0000\u0006, n/a, \u0000\u0000\u0000\u0001, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0001, n/a, \u0000\u0000\u0000\u0002, \u0000\u0000\u0000N, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000\u0013ï]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Kansas, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\fÜ, \u0000\u0000\u000B#, n/a, \u0000\u0000\u0000¬, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0001, \u0000\u0000\u0000\u0002, n/a, \u0000\u0000\u0001), \u0000\u0000\u0001\b, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0000\u001Aß]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Kentucky, \u0000\u0002\u0006K, n/a, \u0000\u0000\u0016@, \u0000\u0000\u0013ª, n/a, \u0000\u0000\u0001u, \u0000\u0000\u0000\u0002, \u0000\u0000\u0000\u0010, \u0000\u0000\u0000\f, n/a, \u0000\u0000\u0004`, \u0000\u0000\u0006o, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u0002<—]"
                            },
                            {
                                "sql_id": "344cf13216c84b621b82d4c212f04b0a",
                                "sql_string": "INSERT INTO cr.fa_bg_checks (year_month, state, permit, permit_recheck, handgun, long_gun, other, multiple, admin, prepawn_handgun, prepawn_long_gun, prepawn_other, redemption_handgun, redemption_long_gun, redemption_other, returned_handgun, returned_long_gun, returned_other, rentals_handgun, rentals_long_gun, private_sale_handgun, private_sale_long_gun, private_sale_other, return_to_seller_handgun, return_to_seller_long_gun, return_to_seller_other, totals) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                                "params": "[2008-06, Louisiana, \u0000\u0000\u0000\u0000, n/a, \u0000\u0000\u0019Í, \u0000\u0000\u0014L, n/a, \u0000\u0000\u0000Ú, \u0000\u0000\u0000\u0000, \u0000\u0000\u0000\u0005, \u0000\u0000\u0000\u0003, n/a, \u0000\u0000\u0002ƒ, \u0000\u0000\u0003@, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, n/a, \u0000\u00004¾]"
                            }
                        ]
                    }
                }
            ]
        }
    ]
}
        "#;
        let result = parse_result(http_result.to_string());
        let remote_ip = match result {
            InboundConnections { ref inbound_connections} => {
                inbound_connections[0].remote_ip.clone()
            },
            _ => String::from("")
        };
        assert_eq!(remote_ip, "127.0.0.1:35692");
        let keyspace = match result {
            InboundConnections { ref inbound_connections} => {
                match &inbound_connections[0].connection_details {
                    Some(details) => {
                        details.cql_connection_details.keyspace.clone()
                    },
                    None => String::from("")
                }
            },
            _ => String::from("")
        };
        assert_eq!(keyspace, "cr");
        let call_details = match result {
            InboundConnections { ref inbound_connections} => {
                match &inbound_connections[0].calls_in_flight {
                    Some(cif) => {
                        cif[0].cql_details.call_details.clone()
                    },
                    None => Vec::<CallDetails>::new()
                }
            },
            _ => Vec::<CallDetails>::new()
        };
        assert_eq!(call_details.len(), 20);
    }
}