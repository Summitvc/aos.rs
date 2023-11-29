#![allow(dead_code)]

use std::io;
use std::sync::{Arc, Mutex};
use std::thread;


use client::*;
use enet_sys::enet_peer_disconnect_now;
use packets::*;

mod client;
mod packets;
mod utils;

fn main() {
    let mut client = Client::init("aos://1931556250:34869", "Crab".to_owned(), GREEN);
    client.log_chat = true;
    client.log_connections = true;
    let admin = true;
    let mut authed: bool = false;
    let mut authid: u8 = 0;

    let shared_input = Arc::new(Mutex::new(String::new()));

    // support for sending messages using stdin
    let _stdin_thread = thread::spawn({
        let shared_input = Arc::clone(&shared_input);
        move || {
            let mut input = String::new();
            loop {
                match io::stdin().read_line(&mut input) {
                    Ok(_) => {
                        let mut shared_input = shared_input.lock().unwrap();
                        *shared_input = input.clone();

                        input.clear();
                    }
                    Err(error) => {
                        eprintln!("Error reading input: {}", error);
                    }
                }
            }
        }
    });

    loop {
        client.service();

        let mut shared_input = shared_input.lock().unwrap();
        let mut input = shared_input.clone();

        if !input.is_empty() {
            input.pop();
            packets::ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL, input);

            shared_input.clear();
        }

        if client.data != [] {
            match client.data[0] {
                STATEDATA => {
                    join(client.peer, client.name.clone(), client.team);
                    if admin == true {
                        //login
                    }
                }
                CHATMESSAGE => {
                    let fields = ChatMessage::deserialize(&client.data);
                    match fields.chatmessage.as_str() {
                        "securepass" => {
                            if authed == false {
                                authed = true;
                                authid = fields.playerid;
                                let hi = format!(
                                    "Welcome, {}!",
                                    client.game.players[fields.playerid as usize].name
                                );
                                ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL, hi);
                            }
                        }
                        "!coords" => {
                            if authid == fields.playerid {
                                let x = client.game.players[client.localplayerid as usize]
                                    .position
                                    .x
                                    .round();
                                let y = client.game.players[client.localplayerid as usize]
                                    .position
                                    .y
                                    .round();
                                let z = client.game.players[client.localplayerid as usize]
                                    .position
                                    .z
                                    .round();

                                let coords = format!("x: {}, y: {}, z: {}", x, y, z);
                                ChatMessage::send(
                                    client.peer,
                                    client.localplayerid,
                                    CHAT_ALL,
                                    coords,
                                );
                            }
                        }
                        "!ls" => {
                            for i in &client.game.players {
                                if i.connected == true {
                                    println!("{}: {}", i.playerid, i.name);
                                }
                            }
                        }
                        x if x.contains("!tp") => {
                            let id: u8 = x.split(" ").collect::<Vec<_>>()[1].parse().unwrap();
                            let target_pos = &client.game.players[id as usize].position;
                            packets::set_position(
                                client.peer,
                                target_pos.x,
                                target_pos.y,
                                target_pos.z,
                            );
                        }
                        x if x.contains("!kill") => {
                                if authid == fields.playerid {
                                    ChatMessage::send(
                                        client.peer,
                                        client.localplayerid,
                                        CHAT_ALL,
                                        "/kill".to_owned(),
                                    );
                                }
                            }
                        "!switch" => {
                            if authid == fields.playerid {
                                let team = client.game.players[client.localplayerid as usize].team;

                                if team == SPECTATOR {
                                    ExtraPackets::change_team(
                                        client.peer,
                                        client.localplayerid,
                                        team + 1,
                                    );
                                } else if team == BLUE {
                                    ExtraPackets::change_team(
                                        client.peer,
                                        client.localplayerid,
                                        team + 1,
                                    );
                                } else if team == GREEN {
                                    ExtraPackets::change_team(
                                        client.peer,
                                        client.localplayerid,
                                        team - 2,
                                    );
                                }
                                packets::ChatMessage::send(
                                    client.peer,
                                    client.localplayerid,
                                    CHAT_ALL,
                                    "Switching team!".to_owned(),
                                );
                            }
                        }
                        "!say" => {
                            let peer = client.peer;
                            let localplayerid = client.localplayerid;
                            ChatMessage::send_lines(
                                &mut client,
                                peer,
                                localplayerid,
                                CHAT_ALL,
                                ["Message 1", "Message 2"].to_vec(),
                            );
                        }
                        "!leave" => {
                            if authid == fields.playerid {
                                unsafe {
                                    enet_peer_disconnect_now(client.peer, 0);
                                    break;
                                }
                            }
                        }
                        "nuke" => unsafe {
                            enet_peer_disconnect_now(client.peer, 0);
                            break;
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
