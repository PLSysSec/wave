#[macro_use]
extern crate dlopen_derive;
mod types;
mod waverunner;
use crate::types::WaveConfig;
use anyhow::{anyhow, Result};
use clap::{App, Arg};
use std::str::FromStr;
use wave::tcb::misc::empty_netlist;
use wave::types::{NetEndpoint, Netlist, WasiProto};

// Converts a space seperated string into a null-seperated Vec<u8>
// also counts the number of words
fn parse_argenv(s: String) -> (Vec<u8>, usize) {
    let mut buf = s.into_bytes();
    let mut inword = false;
    let mut count = 0;
    for c in &mut buf {
        match c {
            0 | 32 => {
                if inword {
                    inword = false;
                    count += 1;
                    *c = 0;
                }
            }
            _ => inword = true,
        }
    }
    (buf, count)
}

// Parses a triple of the form protocol:ip:port
fn parse_net_triple(s: &str) -> Result<NetEndpoint> {
    // let (protocol_s, ip_s, port_s) = s.split(":").collect();
    let triple: Vec<&str> = s.split(':').collect();
    if triple.len() != 3 {
        return Err(anyhow!("Not 3 entries in net_triple: {}", s));
    }
    let protocol_s = triple[0];
    let ip_s = triple[1];
    let port_s = triple[2];

    let protocol = match protocol_s.to_lowercase().as_str() {
        "tcp" => WasiProto::Tcp,
        "udp" => WasiProto::Udp,
        _ => return Err(anyhow!("Unknown protocol: {}", protocol_s)),
    };
    let addr: u32 = std::net::Ipv4Addr::from_str(ip_s)?.into();
    let port = u32::from_str(port_s)?;
    Ok(NetEndpoint {
        protocol,
        addr,
        port,
    })
}

// Parses a comma-seperated string of triples of the form protocol:ip:port
fn parse_netlist(s: String) -> Result<Netlist> {
    let mut netlist = empty_netlist();
    if s.is_empty() {
        return Ok(netlist);
    }
    for (idx, triple_str) in s.split(',').enumerate() {
        if idx >= netlist.len() {
            return Err(anyhow!("Too many net endpoints for the allow list"));
        };
        netlist[idx] = parse_net_triple(triple_str)?;
    }
    Ok(netlist)
}

fn main() {
    let matches = App::new("Wave Runner")
        .version("0.1.0")
        .about("Runs Wasm code -- safely!")
        .arg(
            Arg::new("module path")
                .takes_value(true)
                .help("path to native Wasm module to run")
                .required(true),
        )
        .arg(
            Arg::new("homedir")
                .long("homedir")
                .takes_value(true)
                .help("Home directory")
                .required(true),
        )
        .arg(
            Arg::new("netlist")
                .long("netlist")
                .takes_value(true)
                .help("Allow-list for net endpoints that the Wasm application"),
        )
        .arg(
            Arg::new("args")
                .long("args")
                .takes_value(true)
                .help("Arguments to pass to sandbox (space seperated)"),
        )
        .arg(
            Arg::new("env")
                .long("env")
                .takes_value(true)
                .help("Environment to pass to sandbox (space seperated key-value pairs)"),
        )
        .get_matches();

    let module_path = matches.value_of("module path").unwrap().to_string();
    let homedir = matches.value_of("homedir").unwrap().to_string();
    let netlist_str = matches.value_of("netlist").unwrap_or("").to_string();
    let args_str = matches.value_of("args").unwrap_or("").to_string();
    let env_str = matches.value_of("env").unwrap_or("").to_string();

    let (arg_buffer, argc) = parse_argenv(args_str);
    let (env_buffer, envc) = parse_argenv(env_str);
    let netlist = parse_netlist(netlist_str).unwrap();

    let config = WaveConfig {
        module_path,
        homedir,
        netlist,
        args: arg_buffer,
        argc,
        env: env_buffer,
        envc,
    };

    println!("{:?}", config);

    waverunner::run(&config);
}

