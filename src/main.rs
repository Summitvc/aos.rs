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

        if client.data != vec![0; 0] {
            // vec![0; 0] instead of just [] because serde(required by telegram lib) requires type annotations
            match client.data[0] {
                STATEDATA => {
                    join(client.peer, client.name.clone(), client.team);
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
                        "!switch" => {
                            if authid == fields.playerid {
                                let team = client.game.players[client.localplayerid as usize].team;

                                if team == SPECTATOR {
                                    packets::ChatMessage::send(
                                        client.peer,
                                        client.localplayerid,
                                        CHAT_ALL,
                                        "Switching team!".to_owned(),
                                    );
                                    ExtraPackets::change_team(
                                        client.peer,
                                        client.localplayerid,
                                        team + 1,
                                    );
                                } else if team == BLUE {
                                    ChatMessage::send(
                                        client.peer,
                                        client.localplayerid,
                                        CHAT_ALL,
                                        "Switching team!".to_owned(),
                                    );
                                    ExtraPackets::change_team(
                                        client.peer,
                                        client.localplayerid,
                                        team + 1,
                                    );
                                } else if team == GREEN {
                                    ChatMessage::send(
                                        client.peer,
                                        client.localplayerid,
                                        CHAT_ALL,
                                        "Switching team!".to_owned(),
                                    );
                                    ExtraPackets::change_team(
                                        client.peer,
                                        client.localplayerid,
                                        team - 2,
                                    );
                                }
                            }
                        }
                        "!come" => {
                            if authid == fields.playerid {
                                let id = fields.playerid;
                                ExtraPackets::teleport(client.clone(), id);
                                ChatMessage::send(
                                    client.peer,
                                    client.localplayerid,
                                    CHAT_ALL,
                                    "Coming".to_owned(),
                                );
                            }
                        }
                        "!go" => {
                            let pos = &client.game.players[client.localplayerid as usize].position;
                            let ori =
                                &client.game.players[client.localplayerid as usize].orientation;
                            set_position(client.peer, pos.x + ori.x, pos.y + ori.y, pos.z + ori.z);
                        }
                        "!say" => {
                            ChatMessage::send_lines(
                                client.peer,
                                client.localplayerid,
                                CHAT_ALL,
                                ["Hello everyone", "Im Crab"].to_vec(),
                            );
                        }
                        "!kill" => {
                            if authid == fields.playerid {
                                ChatMessage::send(
                                    client.peer,
                                    client.localplayerid,
                                    CHAT_ALL,
                                    "/kill".to_owned(),
                                );
                            }
                        }
                        "!leave" => {
                            if authid == fields.playerid {
                                unsafe {
                                    enet_peer_disconnect_now(client.peer, 0);
                                    break;
                                }
                            }
                        }
                        _ => {
                            let mut filter = client.data.to_vec(); //fix utf8 invalid byte
                            if client.data[3] == 255 {
                                filter[3] = 0;
                            }
                            if client.data[1] <= 32 {
                                if client.data[2] == 0 {
                                    telegram_notifyrs::send_message(
                                        format!(
                                            "[Global] #{} {}: {}",
                                            fields.playerid,
                                            client.game.players[client.data[1] as usize].name,
                                            &fields.chatmessage
                                        ),
                                        "",
                                        1,
                                    );
                                } else if client.data[2] == 1 {
                                    telegram_notifyrs::send_message(
                                        format!(
                                            "[Team] #{} {}: {}",
                                            fields.playerid,
                                            client.game.players[client.data[1] as usize].name,
                                            &fields.chatmessage
                                        ),
                                        "",
                                        1,
                                    );
                                } else {
                                    telegram_notifyrs::send_message(
                                        format!("[Server] {}", &fields.chatmessage),
                                        "",
                                        1,
                                    );
                                }
                            }
                        }
                    }
                }
                // telegram forwarding
                CREATEPLAYER => {
                    // telegram_notifyrs::send_message(format!("{} connected", client.game.players[client.data[1] as usize].name), "", );
                }
                PLAYERLEFT => {
                    // telegram_notifyrs::send_message(format!("{} disconnected", client.game.players[client.data[1] as usize].name), "", );
                }
                _ => {}
            }
        }
    }
}
