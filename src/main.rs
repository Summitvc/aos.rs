mod utils;
mod packets;

use std::ptr::{null, null_mut};
use std::ffi::{CString, c_void};


use enet_sys::{self, enet_initialize};
use enet_sys::*;

use bincode::{self, Error};

fn main() {
    unsafe{
        enet_initialize();

        let raw_address = utils::ip("aos://16777343:32887").unwrap();
        let ip_addr = CString::new(raw_address[0].as_str()).unwrap().into_raw();
        println!("{:?}", raw_address[0].as_str());
        
        let mut address = ENetAddress{
            host: 0,
            port: 32887
        };
        let mut packet: _ENetPacket  = ENetPacket{
            referenceCount: 0,
            flags: 0,
            data: &mut 0,
            dataLength: 0,
            freeCallback: None,
            userData: std::ptr::null_mut(),
        }; 

        let mut event = ENetEvent{
            type_: 0,
            peer: null_mut(),
            channelID: 0,
            data: 0,
            packet: &mut packet,
        };
        let client = enet_host_create (
            null(),  // address to bind the server host to
            1,        // allow up to 32 clients and/or outgoing connections
            1,         // allow up to 2 channels to be used, 0 and 1
            0,         // assume any amount of incoming bandwidth
            0);        // assume any amount of outgoing bandwidth
        
        // let existingplayer = packets::ExistingPlayer{
        //     player_id:   0,
        //     team:       1,
        //     weapon:     0,
        //     held_item:   0,
        //     kills: 	    0,
        //     blue: 	    0,
        //     green: 	    0,
        //     red: 	    0, 
        //     name: 	    String::from("rust")
        // }; 

        // let mut ExistingPlayerS = bincode::serialize(&existingplayer).unwrap();
        // let len_u64: u64 = ExistingPlayerS.len().try_into().unwrap();
        // let eps_ptr: *const c_void = ExistingPlayerS.as_mut_ptr() as *mut c_void;
        // OPEN WHEN READY

        if client.is_null() {
            panic!("error: client create returned null")
        }
        println!("...ENet client host created");

        enet_host_compress_with_range_coder(client);

        enet_address_set_host(&mut address, CString::from_raw(ip_addr).as_ptr());

        println!("{:?}, {:?}", &address.host, &address.port);

        let peer = enet_host_connect(client, &address, 1, 3);
        if peer.is_null() {
            panic!("Failed to connect to server");
        }

        loop{
            let conn = enet_host_service(client, &mut event, 5000); 

            match conn {
                0 => {},
                1 => {
                    match event.type_ {
                        _ENetEventType_ENET_EVENT_TYPE_RECEIVE => {
                            let data = std::slice::from_raw_parts((*event.packet).data, (*event.packet).dataLength as usize);
                            match data[0] {
                                17 => println!("Chat Message"),
                                18 => println!("Map Start"),
                                15 => println!("Set HP"),
                                19 => println!("Map Chunk"),
                                31 => println!("Map Cached"),
                                _ => {},
                            }
                            enet_packet_destroy(event.packet);
                        },
                        _ => {}
                    }
                },
                _ => { println!("Error servicing ENet: {:?}", conn); break; },
            }
        }
    }
}


// rewrite for clean code