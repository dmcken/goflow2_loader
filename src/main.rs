// Netflow json parser


use std::net::{IpAddr, Ipv4Addr};
use serde::{Deserialize, Serialize};
//use serde_json::Result;
use std::{fs::File, io::{self, BufRead}, path::Path};


#[derive(Serialize, Deserialize, Debug)]
struct NetflowRecord {
    time_received_ns: String, // Date in what looks to be ISO8901
    sequence_num: u64,        // Does this need to be increased?
    time_flow_start_ns: u64,
    time_flow_end_ns: u64,
    bytes: u64,
    packets: u64,
    src_addr: IpAddr,
    dst_addr: IpAddr,
    etype: String,            // IPv4, IPv6, any others? enum?
    proto: String,            // UDP, TCP, others? enum?
    src_port: u16,
    dst_port: u16,
    post_nat_src_ipv4_address: Option<Ipv4Addr>,
    post_nat_dst_ipv4_address: Option<Ipv4Addr>,
    post_napt_src_transport_port: Option<u16>,
    post_napt_dst_transport_port: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JSONNetflowRecord {
    // Define your structure based on the expected JSON format
    /*
    {
        "type":"IPFIX",
        "time_received_ns":"2025-03-15T17:10:51.064235982Z",
        "sequence_num":2259237964,
        "sampling_rate":0,
        "sampler_address":"10.255.255.254",
        "time_flow_start_ns":1742058651000000000,
        "time_flow_end_ns":1742058651000000000,
        "bytes":52,
        "packets":1,
        "src_addr":"10.8.31.235",
        "src_net":"0.0.0.0/0",
        "dst_addr":"1.1.1.1",
        "dst_net":"0.0.0.0/0",
        "etype":"IPv4",
        "proto":"UDP",
        "src_port":41015,
        "dst_port":53,
        "in_if":4,
        "out_if":35,
        "src_mac":"2c:c8:1b:ac:cf:81",
        "dst_mac":"dc:2c:6e:8c:c6:f3",
        "icmp_name":"unknown",
        "post_nat_src_ipv4_address":"6799ef23",
        "post_nat_dst_ipv4_address":"01010101",
        "post_napt_src_transport_port":41015,
        "post_napt_dst_transport_port":53
    }
    */

    r#type: String,           // Looks to be the source (IPFIX), enum?
    time_received_ns: String, // Date in what looks to be ISO8901
    sequence_num: u64,        // Does this need to be increased?
    sampling_rate: u32,       // Most cases should fit in 32-bit
    sampler_address: String,
    time_flow_start_ns: u64,
    time_flow_end_ns: u64,
    bytes: u64,
    packets: u64,
    src_addr: IpAddr,
    src_net: String,
    dst_addr: IpAddr,
    dst_net: String,
    etype: String,            // IPv4, IPv6, any others? enum?
    proto: String,            // UDP, TCP, others? enum?
    src_port: u16,
    dst_port: u16,
    in_if: u16,
    out_if: u16,
    src_mac: String,
    dst_mac: String,
    icmp_name: String,
    post_nat_src_ipv4_address: Option<String>,
    post_nat_dst_ipv4_address: Option<String>,
    post_napt_src_transport_port: Option<u16>,
    post_napt_dst_transport_port: Option<u16>,
}



// if parsed_json.post_nat_src_ipv4_address.is_some() {
//     let raw_str = &parsed_json.post_nat_src_ipv4_address.as_ref(); 
//     let parsed_hex = u32::from_str_radix(
//         &raw_str.unwrap(), 16
//     ).expect("Invalid hex string");
//     let ipv4_addr = Ipv4Addr::from(parsed_hex); 
//     println!("{}", ipv4_addr);
// }

fn main() -> io::Result<()> {
    // Specify the file path
    let path = Path::new("goflow2_20250315_1723.log");

    // Open the file
    let file = File::open(path)?;

    // Create a buffered reader
    let reader = io::BufReader::new(file);

    // Iterate through the lines in the file
    for line in reader.lines() {
        match line {
            Ok(json_line) => {
                // Parse each line as a JSON object
                match serde_json::from_str::<JSONNetflowRecord>(&json_line) {
                    Ok(parsed_json) => {                      
                        
                        let mut current_record = NetflowRecord {
                            time_received_ns:   parsed_json.time_received_ns,
                            sequence_num:       parsed_json.sequence_num,        // Does this need to be increased?
                            time_flow_start_ns: parsed_json.time_flow_start_ns,
                            time_flow_end_ns:   parsed_json.time_flow_end_ns,
                            bytes:              parsed_json.bytes,
                            packets:            parsed_json.packets,
                            src_addr:           parsed_json.src_addr,
                            dst_addr:           parsed_json.dst_addr,
                            etype:              parsed_json.etype,
                            proto: parsed_json.proto,            // UDP, TCP, others? enum?
                            src_port: parsed_json.src_port,
                            dst_port: parsed_json.dst_port,
                            post_nat_src_ipv4_address: None,
                            post_nat_dst_ipv4_address: None,
                            post_napt_src_transport_port: None,
                            post_napt_dst_transport_port: None,
                        };

                        // Print the parsed JSON object
                        // println!("{:?}", parsed_json);
                        println!("{:?}", current_record)
                    },
                    Err(e) => {
                        eprintln!("Failed to parse JSON: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
            }
        }
        break;
    }

    Ok(())
}
