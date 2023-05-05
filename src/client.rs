use crate::packets::*;
use crate::utils;

use std::ffi::{CString};
use std::ptr::{null, null_mut};

use enet_sys::*;
use enet_sys::{self, enet_initialize};

pub const SPECTATOR: i8 = -1;
pub const BLUE: i8 = 0;
pub const GREEN: i8 = 1;

pub struct Game{
    pub players: Vec<Player>,
}

pub struct Client{
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
    pub chat_log: bool,
}


impl Client{
    pub fn init(ip: &str, name: String, team: i8) -> Self{
        unsafe{
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
            let client = enet_host_create(
                null(), 
                1, 
                1, 
                0, 
                0);
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
            let game = Game{
                players
            };
            let data = Vec::new();
            let statedata: StateData = Default::default();
            let localplayerid = 0;
            let chat_log = false;
            
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
                chat_log,
            }
        }
    }

    pub fn service(&mut self){
        unsafe{
                let conn = enet_host_service(self.client, &mut self.event, 0);
                match conn {
                    0 => {self.data = [].to_vec()}
                    1 => match self.event.type_ {
                        _ENetEventType_ENET_EVENT_TYPE_RECEIVE => {
                            let data = std::slice::from_raw_parts(
                                (*self.event.packet).data,
                                (*self.event.packet).dataLength as usize,
                            );
    
                            match data[0] {
                                STATEDATA => {
                                    StateData::deserialize(&mut self.statedata, &mut self.localplayerid, data);

                                    join(self.peer, self.name.clone(), self.team);

                                    self.data = data.to_vec();
                                }
    
                                EXISTINGPLAYER => {
                                    ExistingPlayer::deserialize(data, &mut self.game.players);

                                    self.data = data.to_vec();
                                }
                                
                                WORLDUPDATE => {
                                    WorldUpdate::deserialize(data, &mut self.game.players);

                                    self.data = data.to_vec();
                                }
                                CHATMESSAGE => {
                                    if self.chat_log == true{
                                        let fields = ChatMessage::deserialize(data);
                                        if data[1] <= 32{
                                            println!("{} {}: {}",
                                            fields.playerid,
                                            self.game.players[fields.playerid as usize].name,
                                            fields.chatmessage);
                                        }
                                        else{
                                            println!("{}", ChatMessage::deserialize(data).chatmessage);
                                        }
                                    }

                                    self.data = data.to_vec();
                                }
                                KILLACTION => {
                                    KillAction::deserialize(Default::default(), &mut self.game.players, data);

                                    self.data = data.to_vec();
                                }
                                CREATEPLAYER => {
                                    CreatePlayer::deserialize(Default::default(), &mut self.game.players, data);

                                    self.data = data.to_vec();
                                }
                                PLAYERLEFT => {
                                    self.game.players[data[1] as usize] = Default::default();
                                }
                                _ => {
                                    self.data = data.to_vec();
                                }
                            }
                            enet_packet_destroy(self.event.packet);
                        }
                        _ => {
                            // println!("undefined packet type: {:?}", self.event.type_);
                        }
                    }
                    _ => {
                        println!("Error servicing ENet: {:?}", conn);
                    }
                }
            }
        }
        pub fn disconnect(&self){
            unsafe{
                enet_peer_disconnect(self.peer, 0);
            }
        }
    }
