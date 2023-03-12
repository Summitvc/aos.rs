use std::ffi::CString;
mod utils;
use std::ptr::{null, null_mut};

use enet_sys::{self, enet_initialize};
use enet_sys::*;

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
        let mut event = ENetEvent{
            type_: 0,
            peer: null_mut(),
            channelID: 0,
            data: 0,
            packet: null_mut(),
        };
        let client = enet_host_create (
            null(),  // address to bind the server host to
            32,        // allow up to 32 clients and/or outgoing connections
            1,         // allow up to 2 channels to be used, 0 and 1
            0,         // assume any amount of incoming bandwidth
            0);        // assume any amount of outgoing bandwidth

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


        if enet_host_service(client, &mut event, 5000)  > 0 && 
        event.type_ == _ENetEventType_ENET_EVENT_TYPE_CONNECT{
            println!("Connection succeeded, {:?}", event.packet);
        }
        else{
            enet_peer_reset(peer);
        }
    }
}
/*
    TODO: FIX UTILS. MAKE ENetAddress accept from ip function

 */