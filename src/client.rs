use crate::packets::{self, ExistingPlayer, WorldUpdate};
use crate::packets::Player;
use crate::utils;

use std::ffi::{c_void, CString};
use std::ptr::{null, null_mut};

use enet_sys::*;
use enet_sys::{self, enet_initialize};

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
    pub data: Vec<u8>,
    pub statedata: packets::StateData,
    pub chat_log: bool,
}


impl Client{
    pub fn init(ip: &str, name: String) -> Self{
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
            let mut event: _ENetEvent = ENetEvent {
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

            let statedata: packets::StateData = Default::default();

            let chat_log = false;
            
            println!("Connecting...");
            
            Client { 
                client, 
                peer, 
                event, 
                packet,
                game,
                name,
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
                                packets::STATEDATA => {
                                    packets::StateData::deserialize_join(&mut self.statedata, 
                                        data, 
                                        self.name.clone(), 
                                        self.peer, 
                                        &mut self.game.players);

                                    self.data = data.to_vec();
                                }
    
                                packets::EXISTINGPLAYER => {
                                    ExistingPlayer::deserialize(data, &mut self.game.players);
                                    self.data = data.to_vec();
                                }
                                
                                packets::WORLDUPDATE => {
                                    WorldUpdate::deserialize(data, &mut self.game.players);
                                    self.data = data.to_vec();
                                }
                                packets::CHATMESSAGE => {
                                    if self.chat_log == true{
                                        if data[1] <= 32{
                                            println!("{}: {}", 
                                            self.game.players[data[1] as usize].name,
                                            packets::ChatMessage::deserialize(data).chatmessage);
                                        }
                                        else{
                                            println!("{}", packets::ChatMessage::deserialize(data).chatmessage);
                                        }
                                    }
                                    else{

                                    }
                                    self.data = data.to_vec();
                                }
                                _ => {
                                    self.data = data.to_vec();
                                    enet_packet_destroy(self.event.packet);
                                }
                            }
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