use crate::packets;
use crate::packets::*;
use crate::utils;

use std::ffi::CString;
use std::ptr::{null, null_mut};

use enet_sys::*;
use enet_sys::{self, enet_initialize};

use colored::Colorize;

pub const SPECTATOR: i8 = -1;
pub const BLUE: i8 = 0;
pub const GREEN: i8 = 1;

#[derive(Clone)]
pub struct Game {
    pub players: Vec<Player>,
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
            let client = enet_host_create(null(), 1, 1, 0, 0);
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
            let game = Game { players };
            let data = Vec::new();
            let statedata: StateData = Default::default();
            let localplayerid = 0;
            let log_chat = false;
            let log_connections = false;

            println!("Connecting...");

            Client {
                client,
                peer,
                event,
                packet,
                game,
                name,
                localplayerid,
                team,
                data,
                statedata,
                log_chat,
                log_connections,
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
                            STATEDATA => {
                                StateData::deserialize(
                                    &mut self.statedata,
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
