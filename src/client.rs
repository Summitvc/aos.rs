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
    pub exps: Vec<u8>,
    pub exps_ptr: *const c_void
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

            let mut exps: Vec<u8> = ExistingPlayer::serialize(&Default::default(), name.clone());
            let exps_ptr: *const c_void = exps.as_mut_ptr() as *mut c_void;

            Client { 
                client, 
                peer, 
                event, 
                packet,
                game,
                name,
                data,
                exps,
                exps_ptr
            }
        }
    }

    pub fn service(&mut self){
        unsafe{
                let conn = enet_host_service(self.client, &mut self.event, 0);
                match conn {
                    0 => {}
                    1 => match self.event.type_ {
                        _ENetEventType_ENET_EVENT_TYPE_RECEIVE => {
                            let data = std::slice::from_raw_parts(
                                (*self.event.packet).data,
                                (*self.event.packet).dataLength as usize,
                            );
    
                            match data[0] {
                                packets::STATEDATA => {
                                    let new_packet = enet_packet_create(
                                        self.exps_ptr,
                                        self.exps.len() as u64,
                                        _ENetPacketFlag_ENET_PACKET_FLAG_RELIABLE,
                                    );
                                    enet_peer_send(self.peer, 0, new_packet);
                                }
    
                                packets::EXISTINGPLAYER => {
                                    ExistingPlayer::deserialize(data, &mut self.game.players);
                                }
    
                                packets::WORLDUPDATE => {
                                    WorldUpdate::deserialize(data, &mut self.game.players);
                                }
                                _ => {
                                    self.data = data.to_vec();
                                }
                            }
                            enet_packet_destroy(self.event.packet);
                        }
                        _ => {
                            println!("undefined packet type: {:?}", self.event.type_);
                        }
                    },
                    _ => {
                        println!("Error servicing ENet: {:?}", conn);
                    }
                }
            }
        }
    }