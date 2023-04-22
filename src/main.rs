mod packets;
mod utils;

use std::default;
use std::ffi::{c_void, CString};
use std::ptr::{null, null_mut};

use enet_sys::*;
use enet_sys::{self, enet_initialize};

use crate::packets::{Player};

fn main() {
    unsafe {
        enet_initialize();

        //let raw_address = utils::ip("aos://16777343:32887").unwrap();
        let raw_address = utils::ip("aos://1931556250:34869").unwrap();
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
        let mut event = ENetEvent {
            type_: 0,
            peer: null_mut(),
            channelID: 0,
            data: 0,
            packet: &mut packet,
        };

        let client = enet_host_create(
            null(), // address to bind the server host to
            1,      // allow up to 32 clients and/or outgoing connections
            1,      // allow up to 2 channels to be used, 0 and 1
            0,      // assume any amount of incoming bandwidth
            0,
        ); // assume any amount of outgoing bandwidth

        let existingplayer = packets::ExistingPlayer {
            playerid: 0,
            team: 0,
            weapon: 0,
            helditem: 0,
            kills: 0,
            blue: 0,
            green: 0,
            red: 0,
            name: String::from("Crab"),
        };

        let mut players: Vec<packets::Player> = vec![Default::default(); 32];

        let mut player = Player::new();

        let mut exps = existingplayer.serialize("Crab".to_owned());
        println!("{:?}", exps);
        let exps_ptr: *const c_void = exps.as_mut_ptr() as *mut c_void;

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

        loop {
            let conn = enet_host_service(client, &mut event, 5000);

            match conn {
                0 => {}
                1 => match event.type_ {
                    _ENetEventType_ENET_EVENT_TYPE_RECEIVE => {
                        let data = std::slice::from_raw_parts(
                            (*event.packet).data,
                            (*event.packet).dataLength as usize,
                        );
                        match data[0] {
                            packets::STATEDATA => {
                                println!("State Data");
                                let new_packet = enet_packet_create(
                                    exps_ptr,
                                    exps.len() as u64,
                                    _ENetPacketFlag_ENET_PACKET_FLAG_RELIABLE,
                                );
                                enet_peer_send(peer, 0, new_packet);
                            }
                            packets::EXISTINGPLAYER => {

                                packets::deserialize_ep(data, &mut player);
                                
                                players[player.playerid.clone() as usize] = player.clone();

                                for player in &players{
                                    if player.name != ""{
                                        println!("{}", player.name);
                                    }
                                }
                            }
                            //packets::WORLDUPDATE => println!("World Update"),
                            packets::KILLACTION => println!("Kill Action"),
                            packets::CHATMESSAGE => println!("Chat Message"),
                            packets::MAPSTART => println!("Map Start"),
                            packets::MAPCHUNK => println!("Map Chunk"),
                            packets::MAPCACHED => println!("Map Cached"),
                            2 => {},
                            _ => println!("{:?}", data[0]),
                        }
                        enet_packet_destroy(event.packet);
                    }
                    _ => {}
                },
                _ => {
                    println!("Error servicing ENet: {:?}", conn);
                    break;
                }
            }
        }
    }
}

// rewrite for clean code
