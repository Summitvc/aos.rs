use crate::packets;
use crate::packets::*;
use crate::utils;

use std::ffi::CString;
use std::io::Read;
use std::ptr::{null, null_mut};

use enet_sys::enet_initialize;
use enet_sys::*;

use bit_vec::BitVec;
use flate2::read::ZlibDecoder;

use colored::Colorize;

pub const SPECTATOR: i8 = -1;
pub const BLUE: i8 = 0;
pub const GREEN: i8 = 1;

#[derive(Clone)]
pub struct Game {
    pub players: Vec<Player>,
    pub map: WorldMap,
}

#[derive(Clone)]
pub struct Client {
    pub client: *mut _ENetHost,
    pub peer: *mut _ENetPeer,
    pub event: _ENetEvent,
    pub packet: _ENetPacket,
    pub game: Game,
    pub name: String,
    pub localplayerid: u8,
    pub team: i8,
    pub data: Vec<u8>,
    pub statedata: StateData,
    pub log_chat: bool,
    pub log_connections: bool,
}

// fix for #[allow(non_upper_case_globals)] not working
pub const ENET_TYPE_RECIEVE: u32 = _ENetEventType_ENET_EVENT_TYPE_RECEIVE;

impl Client {
    pub fn init(ip: &str, name: String, team: i8) -> Self {
        unsafe {
            enet_initialize();
            let raw_address = utils::ip(ip).unwrap();
            let ip_addr = CString::new(raw_address[0].as_str()).unwrap().into_raw();

            let mut address = ENetAddress {
                host: 0,
                port: raw_address[1].parse::<u16>().unwrap(),
            };
            let mut packet: _ENetPacket = ENetPacket {
                referenceCount: 0,
                flags: 0,
                data: &mut 0,
                dataLength: 0,
                freeCallback: None,
                userData: std::ptr::null_mut(),
            };
            let event: _ENetEvent = ENetEvent {
                type_: 0,
                peer: null_mut(),
                channelID: 0,
                data: 0,
                packet: &mut packet,
            };
            let client = enet_host_create(null(), 1, 2, 0, 0);
            if client.is_null() {
                panic!("error: client create returned null")
            }
            println!("...ENet client host created");

            enet_host_compress_with_range_coder(client);
            enet_address_set_host(&mut address, CString::from_raw(ip_addr).as_ptr());

            let peer = enet_host_connect(client, &address, 1, 3);
            if peer.is_null() {
                panic!("Failed to connect to server");
            }

            let players: Vec<Player> = vec![Default::default(); 32];

            let map = WorldMap {
                data: Vec::new(),
                blocks: BitVec::from_elem((X_SIZE * Y_SIZE * Z_SIZE) as usize, false),
                colors: vec![vec![vec![Color::default(); Z_SIZE as usize]; Y_SIZE as usize]; X_SIZE as usize],
            };

            println!("Connecting...");

            Client {
                client,
                peer,
                event,
                packet,
                game: Game { players, map },
                name,
                localplayerid: 0,
                team,
                data: Vec::new(),
                statedata: Default::default(),
                log_chat: false,
                log_connections: false,
            }
        }
    }

    pub fn service(&mut self) {
        unsafe {
            let conn = enet_host_service(self.client, &mut self.event, 1);
            match conn {
                0 => self.data = [].to_vec(),
                1 => match self.event.type_ {
                    ENET_TYPE_RECIEVE => {
                        let data = std::slice::from_raw_parts(
                            (*self.event.packet).data,
                            (*self.event.packet).dataLength as usize,
                        );
                        match data[0] {
                            MAPSTART => {
                                self.game.map.data = Vec::new();
                                self.game.map.blocks = BitVec::from_elem((X_SIZE * Y_SIZE * Z_SIZE) as usize, false);
                                self.game.map.colors = vec![vec![vec![Color::default(); Z_SIZE as usize]; Y_SIZE as usize]; X_SIZE as usize];
                            }
                            MAPCHUNK => {
                                self.game.map.data.append(&mut data[1..].to_vec());
                            }
                            STATEDATA => {
                                let mut d = ZlibDecoder::new(self.game.map.data.as_slice());
                                let mut buf: Vec<u8> = Vec::new();
                                d.read_to_end(buf.as_mut()).unwrap();
                                self.game.map.data.clear();
                                self.game.map.data = buf;

                                WorldMap::deserialize(&mut self.game.map);

                                StateData::deserialize(
                                    &mut self.statedata,
                                    &mut self.game.players,
                                    &mut self.localplayerid,
                                    data,
                                );
                            }
                            EXISTINGPLAYER => {
                                ExistingPlayer::deserialize(data, &mut self.game.players);
                            }
                            WORLDUPDATE => {
                                WorldUpdate::deserialize(data, &mut self.game.players);
                            }
                            HANDSHAKE_INIT => {
                                let mut buf: Vec<u8> = vec![];
                                buf.push(HANDSHAKE_RETURN);
                                buf.extend_from_slice(&data[1..]);
                                packets::send(self.peer, buf);
                            }
                            VERSION_REQ => {
                                let version = VersionInfo {
                                    client: i8::from_le_bytes("r".as_bytes().try_into().unwrap()),
                                    version_major: 1,
                                    version_minor: 2,
                                    version_revision: 5,
                                    operating_system_info: "aos.rs".to_string(),
                                };

                                let mut buf: Vec<u8> = vec![];

                                buf.push(VERSION_RESP);
                                buf.push(version.client as u8);
                                buf.push(version.version_major);
                                buf.push(version.version_minor);
                                buf.push(version.version_revision);
                                buf.extend_from_slice(version.operating_system_info.as_bytes());

                                packets::send(self.peer, buf);
                            }

                            CHATMESSAGE => {
                                if self.log_chat == true {
                                    if data[1] <= 32 {
                                        if data[2] == 0 {
                                            let fields = ChatMessage::deserialize(&data);
                                            println!(
                                                "[Global] {}{} {}: {}",
                                                "#".white(), //ew
                                                fields.playerid.to_string().white(),
                                                self.game.players[fields.playerid as usize]
                                                    .name
                                                    .purple()
                                                    .italic(),
                                                fields.chatmessage.green().bold()
                                            );
                                        } else if data[2] == 1 {
                                            let fields = ChatMessage::deserialize(&data);
                                            println!(
                                                "{} {}{} {}: {}",
                                                "[Team]".bright_blue(),
                                                "#".white(),
                                                fields.playerid.to_string().white(),
                                                self.game.players[fields.playerid as usize]
                                                    .name
                                                    .purple()
                                                    .italic(),
                                                fields.chatmessage.green().bold()
                                            );
                                        } else {
                                            let fields = ChatMessage::deserialize(&data);
                                            println!(
                                                "{} {}",
                                                "[Server]".strikethrough(),
                                                fields.chatmessage.white().bold()
                                            );
                                        }
                                    }
                                }
                            }
                            KILLACTION => {
                                KillAction::deserialize(
                                    Default::default(),
                                    &mut self.game.players,
                                    data,
                                );
                            }
                            CREATEPLAYER => {
                                if self.game.players[data[1] as usize].connected == false
                                    && data[1] != self.localplayerid
                                    && self.log_connections == true
                                {
                                    let mut template = format!("/client #{}", data[1]);
                                    CreatePlayer::deserialize(
                                        Default::default(),
                                        &mut self.game.players,
                                        data,
                                    );

                                    packets::ChatMessage::send(
                                        self.peer,
                                        self.localplayerid,
                                        CHAT_ALL,
                                        template.clone(),
                                    );
                                    println!(
                                        "{} {}",
                                        self.game.players[data[1] as usize].name.bright_blue(),
                                        "joined".bright_blue()
                                    );

                                    template.clear();
                                } else {
                                    CreatePlayer::deserialize(
                                        Default::default(),
                                        &mut self.game.players,
                                        data,
                                    );
                                }
                            }
                            PLAYERLEFT => {
                                if data[1] != self.localplayerid && self.log_connections == true {
                                    println!(
                                        "{} {}",
                                        self.game.players[data[1] as usize].name.bright_red(),
                                        "disconnected".bright_red()
                                    );
                                }
                                self.game.players[data[1] as usize].connected = false;
                            }
                            _ => {}
                        }
                        self.data = data.to_vec();
                        enet_packet_destroy(self.event.packet);
                    }
                    _ => {
                        // println!("undefined packet type: {:?}", self.event.type_);
                    }
                },
                _ => {
                    println!("Error servicing ENet: {:?}", conn);
                }
            }
        }
    }
}
