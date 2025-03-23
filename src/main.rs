// Netflow json parser

// Standard library
use std::{fs::File, io::{self, BufRead}, path::Path};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};

// External
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Connection, Error, PgConnection, PgPool, Postgres};
use tracing::{info, debug, warn, error, span, Level};
use tracing_subscriber;


// Logger

#[derive(Debug)]
struct NetflowRecord {
    time_received_ns: DateTime<Utc>,
    sequence_num: i64,        
    time_flow_start_ns: i64,
    time_flow_end_ns: i64,
    bytes: i64,
    packets: i64,
    src_addr: IpAddr,
    dst_addr: IpAddr,
    etype: i32,
    proto: i16,
    src_port: i32,
    dst_port: i32,
    post_nat_src_ipv4_address: Option<IpAddr>,
    post_nat_dst_ipv4_address: Option<IpAddr>,
    post_napt_src_transport_port: Option<i32>,
    post_napt_dst_transport_port: Option<i32>,
}

// These fields are defined here:
// https://github.com/netsampler/goflow2/blob/main/docs/protocols.md
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
    sequence_num: i64,        // Does this need to be increased?
    sampling_rate: u32,       // Most cases should fit in 32-bit
    sampler_address: String,
    time_flow_start_ns: i64,
    time_flow_end_ns: i64,
    bytes: i64,
    packets: i64,
    src_addr: IpAddr,
    src_net: String,
    dst_addr: IpAddr,
    dst_net: String,
    etype: String,            // IPv4, IPv6, any others? enum?
    proto: String,            // UDP, TCP, others? enum?
    src_port: i32,
    dst_port: i32,
    in_if: u16,
    out_if: u16,
    src_mac: String,
    dst_mac: String,
    icmp_name: String,
    post_nat_src_ipv4_address: Option<String>,
    post_nat_dst_ipv4_address: Option<String>,
    post_napt_src_transport_port: Option<i32>,
    post_napt_dst_transport_port: Option<i32>,
}

// Ethernet Protocol Mapping
// https://github.com/netsampler/goflow2/blob/main/producer/proto/render.go - etypeName (convert to decimal)
// Which is derived from:
// https://www.iana.org/assignments/ieee-802-numbers/ieee-802-numbers.xhtml#ieee-802-numbers-1
fn create_ethernet_protocol_map() -> HashMap<String, i32> { 
    let protocol_list_str = r#"
2054:ARP
2048:IPv4
34525:IPv6
    "#;
    let protocol_list_lines: Vec<String> = protocol_list_str
        .lines() // Split by newlines
        .map(|line| line.trim().to_string()) 
        .filter(|line| !line.is_empty()) 
        .collect();
    let mut protocol_map: HashMap<String, i32> = HashMap::new();
    

    for line in protocol_list_lines {
        let parts: Vec<String> = line.split(':').map(|s| s.trim().to_string()).collect();
        let protocol_id_parse: Result<i32, _> = parts[0].parse();
        let protocol_id: i32;
        match protocol_id_parse {
            Ok(value) => protocol_id = value,
            Err(e) => {
                println!("Failed to parse ethernet protocol ID: {} => {:?}", e, parts);
                continue;
            }
        }
        protocol_map.insert(
            parts[1].to_string().clone(),
            protocol_id
        );
    }

    // println!("{:?}",protocol_map);
    return  protocol_map;
}

// IP Protocol Mapping
// https://github.com/netsampler/goflow2/blob/main/producer/proto/render.go - protoName
fn create_protocol_map() -> HashMap<String,i16> {
    let protocol_list_str = r#"
		0:   HOPOPT
		1:   ICMP
		2:   IGMP
		3:   GGP
		4:   IPv4
		5:   ST
		6:   TCP
		7:   CBT
		8:   EGP
		9:   IGP
		10:  BBN-RCC-MON
		11:  NVP-II
		12:  PUP
		13:  ARGUS
		14:  EMCON
		15:  XNET
		16:  CHAOS
		17:  UDP
		18:  MUX
		19:  DCN-MEAS
		20:  HMP
		21:  PRM
		22:  XNS-IDP
		23:  TRUNK-1
		24:  TRUNK-2
		25:  LEAF-1
		26:  LEAF-2
		27:  RDP
		28:  IRTP
		29:  ISO-TP4
		30:  NETBLT
		31:  MFE-NSP
		32:  MERIT-INP
		33:  DCCP
		34:  3PC
		35:  IDPR
		36:  XTP
		37:  DDP
		38:  IDPR-CMTP
		39:  TP++
		40:  IL
		41:  IPv6
		42:  SDRP
		43:  IPv6-Route
		44:  IPv6-Frag
		45:  IDRP
		46:  RSVP
		47:  GRE
		48:  DSR
		49:  BNA
		50:  ESP
		51:  AH
		52:  I-NLSP
		53:  SWIPE
		54:  NARP
		55:  Min-IPv4
		56:  TLSP
		57:  SKIP
		58:  IPv6-ICMP
		59:  IPv6-NoNxt
		60:  IPv6-Opts
		61:  any-host-internal-protocol
		62:  CFTP
		63:  any-local-network
		64:  SAT-EXPAK
		65:  KRYPTOLAN
		66:  RVD
		67:  IPPC
		68:  any-distributed-file-system
		69:  SAT-MON
		70:  VISA
		71:  IPCV
		72:  CPNX
		73:  CPHB
		74:  WSN
		75:  PVP
		76:  BR-SAT-MON
		77:  SUN-ND
		78:  WB-MON
		79:  WB-EXPAK
		80:  ISO-IP
		81:  VMTP
		82:  SECURE-VMTP
		83:  VINES
		84:  IPTM
		85:  NSFNET-IGP
		86:  DGP
		87:  TCF
		88:  EIGRP
		89:  OSPFIGP
		90:  Sprite-RPC
		91:  LARP
		92:  MTP
		93:  AX.25
		94:  IPIP
		95:  MICP
		96:  SCC-SP
		97:  ETHERIP
		98:  ENCAP
		99:  any-private-encryption-scheme
		100: GMTP
		101: IFMP
		102: PNNI
		103: PIM
		104: ARIS
		105: SCPS
		106: QNX
		107: A/N
		108: IPComp
		109: SNP
		110: Compaq-Peer
		111: IPX-in-IP
		112: VRRP
		113: PGM
		114: any-0-hop-protocol
		115: L2TP
		116: DDX
		117: IATP
		118: STP
		119: SRP
		120: UTI
		121: SMP
		122: SM
		123: PTP
		124: ISIS over IPv4
		125: FIRE
		126: CRTP
		127: CRUDP
		128: SSCOPMCE
		129: IPLTpostgres bulk insert
		131: PIPE
		132: SCTP
		133: FC
		134: RSVP-E2E-IGNORE
		135: Mobility Header
		136: UDPLite
		137: MPLS-in-IP
		138: manet
		139: HIP
		140: Shim6
		141: WESP
		142: ROHC
		143: Ethernet
		144: AGGFRAG
		145: NSH
    "#;
    let protocol_list_lines: Vec<String> = protocol_list_str
        .lines() // Split by newlines
        .map(|line| line.trim().to_string()) 
        .filter(|line| !line.is_empty()) 
        .collect();
    let mut protocol_map: HashMap<String, i16> = HashMap::new();
    

    for line in protocol_list_lines {
        let parts: Vec<String> = line.split(':')
            .map(|s| s.trim().to_string())
            .collect();
        let protocol_id_parse: Result<i16, _> = parts[0].parse();
        let protocol_id: i16;
        match protocol_id_parse {
            Ok(value) => protocol_id = value,
            Err(e) => {
                println!("Failed to parse protocol ID: {} => {:?}", e, parts);
                continue;
            }
        }
        protocol_map.insert(
            parts[1].to_string().clone(),
            protocol_id
        );
    }

    // println!("{:?}",protocol_map);
    return  protocol_map;
}

fn parse_hex_ipv4(hex_str: String) -> Ipv4Addr {
    let parsed_hex = u32::from_str_radix(
        &hex_str, 16
    ).expect("Invalid hex string");
    let ipv4_addr = Ipv4Addr::from(parsed_hex);

    return ipv4_addr;
}

fn parse_json_record(parsed_json: &JSONNetflowRecord,
                     protocol_map: &HashMap<String,i16>, 
                     ethernet_map: &HashMap<String,i32>) -> NetflowRecord {

    let time_received_ns_str = parsed_json.time_received_ns.clone();
    let time_received_ns =  DateTime::parse_from_rfc3339(
            time_received_ns_str.as_str(), 
            // "%Y-%m-%d %H:%M:%S%.9f"
        )
        .expect(format!("Failed to parse date string: {}", time_received_ns_str).as_str())
        .with_timezone(&Utc);


    let post_nat_src_ipv4_address = parsed_json.post_nat_src_ipv4_address.clone();
    let post_nat_dst_ipv4_address = parsed_json.post_nat_dst_ipv4_address.clone();

    let mut post_nat_src_ipv4_str: Option<IpAddr> = None;
    let mut post_nat_dst_ipv4_str: Option<IpAddr> = None;

    if parsed_json.post_nat_src_ipv4_address.is_some() {
        post_nat_src_ipv4_str = Some(IpAddr::V4(parse_hex_ipv4(
            post_nat_src_ipv4_address.unwrap()
        )));
    }

    if parsed_json.post_nat_dst_ipv4_address.is_some() {
        post_nat_dst_ipv4_str = Some(IpAddr::V4(parse_hex_ipv4(
            post_nat_dst_ipv4_address.unwrap()
        )));
    }

    let protocol_name = parsed_json.proto.clone();
    let protocol_id: i16;
    match protocol_map.get(&protocol_name) {
        Some(proto_id) => { 
            protocol_id = proto_id.clone(); 
        }
        None => {
            println!("Unknown protocol: '{:?}'", parsed_json);
            protocol_id = -1;
        }
    }
    let ethernet_protocol_name = parsed_json.etype.clone();
    let ethernet_protocol_id: i32;
    match ethernet_map.get(&ethernet_protocol_name) {
        Some(ether_proto_id) => {
            ethernet_protocol_id = ether_proto_id.clone(); 
        }
        None => {
            println!("Unknown ethernet protocol: {}", ethernet_protocol_name);
            ethernet_protocol_id = -1;
        }
    }
    
    let current_record = NetflowRecord {
        time_received_ns:   time_received_ns,
        sequence_num:       parsed_json.sequence_num.clone(),        // Does this need to be increased?
        time_flow_start_ns: parsed_json.time_flow_start_ns.clone(),
        time_flow_end_ns:   parsed_json.time_flow_end_ns.clone(),
        bytes:              parsed_json.bytes.clone(),
        packets:            parsed_json.packets.clone(),
        src_addr:           parsed_json.src_addr.clone(),
        dst_addr:           parsed_json.dst_addr.clone(),
        etype:              ethernet_protocol_id,
        proto:              protocol_id,
        src_port:           parsed_json.src_port.clone(),
        dst_port:           parsed_json.dst_port.clone(),
        post_nat_src_ipv4_address: post_nat_src_ipv4_str,
        post_nat_dst_ipv4_address: post_nat_dst_ipv4_str,
        post_napt_src_transport_port: parsed_json.post_napt_src_transport_port.clone(),
        post_napt_dst_transport_port: parsed_json.post_napt_dst_transport_port.clone(),
    };
    return current_record;
}

// fn connect_db() -> Pool<Postgres> {
//     let hostname = "192.168.1.60";
//     let username = "netflow";
//     let password = "6C5fcjnmwPCdw36VmA24";

//     let pool = PgPoolOptions::new()
//         .max_connections(5)
//         .connect("postgres://netflow:6C5fcjnmwPCdw36VmA24@192.168.1.60/netflow");

//     return pool;
// }

async fn insert_flow(db_obj: &mut sqlx::Transaction<'_, sqlx::Postgres>,
                     current_record: &NetflowRecord) -> Result<(), Error> {

    let query = "
        INSERT INTO public.flows (
            time_received_ns,
            sequence_num,        
            time_flow_start_ns,
            time_flow_end_ns,
            bytes,
            packets,
            src_addr,
            dst_addr,
            etype,
            proto,
            src_port,
            dst_port,
            post_nat_src_ipv4_address,
            post_nat_dst_ipv4_address,
            post_napt_src_transport_port,
            post_napt_dst_transport_port
        ) VALUES (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            $7,
            $8,
            $9,
            $10,
            $11,
            $12,
            $13,
            $14,
            $15,
            $16
        )
    ";

    sqlx::query(query)
        .bind(&current_record.time_received_ns)   
        .bind(&current_record.sequence_num)
        .bind(&current_record.time_flow_start_ns)
        .bind(&current_record.time_flow_end_ns)
        .bind(&current_record.bytes)
        .bind(&current_record.packets)
        .bind(&current_record.src_addr)
        .bind(&current_record.dst_addr)
        .bind(&current_record.etype)
        .bind(&current_record.proto)
        .bind(&current_record.src_port)
        .bind(&current_record.dst_port)
        .bind(&current_record.post_nat_src_ipv4_address)
        .bind(&current_record.post_nat_dst_ipv4_address)
        .bind(&current_record.post_napt_src_transport_port)
        .bind(&current_record.post_napt_dst_transport_port)
        .execute( &mut **db_obj)
        .await
        .expect("Failed to insert row");

    // println!("Inserted flow: {:?}", current_record);


    Ok(())
}


#[async_std::main] 
async fn main() ->  Result<(), sqlx::Error>  {
    tracing_subscriber::fmt::init();
    info!("Starting");

    let db_url = "postgres://netflow:6C5fcjnmwPCdw36VmA24@192.168.1.60/netflow";
    let mut pg_connection = PgConnection::connect(db_url)
        .await
        .expect("Failed to connect to the database");
    let mut pg_transaction = pg_connection.begin()
        .await
        .expect("Failed to begin transaction");

    let protocol_map = create_protocol_map();
    let ethernet_map = create_ethernet_protocol_map();

    // JSON file to process
    let path = Path::new("goflow2_20250315_1723.log");
    let file = File::open(path)?;
    let reader = io::BufReader::with_capacity(
        16 * 1024 * 1024, // 16MB bufffer 
        file
    );

    // Loop variables
    let mut parsed_vec: Vec<NetflowRecord> = Vec::new();
    let mut counter: u32 = 0;

    // Iterate through the lines in the file
    for line in reader.lines() {
        match line {
            Ok(json_line) => {
                // Parse each line as a JSON object
                match serde_json::from_str::<JSONNetflowRecord>(&json_line) {
                    Ok(parsed_json) => {  
                        let current_record = parse_json_record( 
                            &parsed_json,
                            &protocol_map,
                            &ethernet_map,
                        );
                        if current_record.proto == -1 {
                            continue;
                        }
                        // println!("{:?}", current_record);
                        // insert_flow(&mut pg_transaction, &current_record).await?;
                        parsed_vec.push(current_record);
                    },
                    Err(e) => {
                        eprintln!("Failed to parse JSON: {}", e);
                    }
                }
                counter = counter + 1;
                if counter % 50000 == 0 {
                    info!("Processed rows: {}", counter);
                    // Close the transaction and restart it
                    pg_transaction.commit().await.expect("Failed to commit transaction");
                    pg_transaction = pg_connection.begin().await
        .expect("Failed to begin transaction");
                }
            }
            Err(e) => {
                eprintln!("Error reading line: {}", e);
            }
        }
        // break;
    }
    pg_transaction.commit().await.expect("Failed to commit transaction");

    info!("Done");
    Ok(())
}
