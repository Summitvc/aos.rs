#![allow(dead_code)]

use std::io;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use client::*;
use enet_sys::{enet_deinitialize, enet_peer_disconnect_now};
use packets::*;

mod client;
mod packets;
mod utils;

use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut client = Client::init("aos://3782433610:32000", "Deuce".to_owned(), GREEN);
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

        if client.data != [] {
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
                        "!draw" => {
                            let mut img = image::RgbImage::new(X_SIZE as u32, Y_SIZE as u32);
                            for x in 0..X_SIZE {
                                for y in 0..Y_SIZE {
                                    let top_block =
                                        WorldMap::get_top_block(x, y, &mut client.game.map);
                                    img.put_pixel(
                                        x as u32,
                                        y as u32,
                                        image::Rgb([
                                            client.game.map.colors[x as usize][y as usize]
                                                [top_block as usize]
                                                .red,
                                            client.game.map.colors[x as usize][y as usize]
                                                [top_block as usize]
                                                .green,
                                            client.game.map.colors[x as usize][y as usize]
                                                [top_block as usize]
                                                .blue,
                                        ]),
                                    );
                                }
                            }
                            img.save("map.png").unwrap();
                        }
                        "!ls" => {
                            for i in &client.game.players {
                                if i.connected == true {
                                    println!("{}: {}", i.playerid, i.name);
                                }
                            }
                        }
                        "!get" => {
                            for i in client.game.players.clone() {
                                if i.connected == true {
                                    let message = format!("/client #{}", i.playerid);
                                    let mut g = true;

                                    let (tx, rx) = mpsc::channel();
                                    loop {
                                        if g == true {
                                            ChatMessage::send(
                                                client.peer,
                                                client.localplayerid,
                                                CHAT_ALL,
                                                message.to_owned(),
                                            );

                                            let tx_clone = tx.clone();

                                            thread::spawn(move || {
                                                thread::sleep(Duration::from_secs(2));
                                                tx_clone.send(()).unwrap();
                                            });
                                        }
                                        g = false;
                                        client.service();
                                        if client.data != [] {
                                            match client.data[0] {
                                                CHATMESSAGE => {
                                                    let fields =
                                                        ChatMessage::deserialize(&client.data);
                                                    match fields.chatmessage.as_str() {
                                                        x if x.contains("connected with") => {
                                                            let mut cl: String = x.to_string();
                                                            cl.push_str("\n");
                                                            let mut file = File::options()
                                                                .append(true)
                                                                .create(true)
                                                                .open("clients.txt")
                                                                .unwrap();
                                                            file.write_all(cl.as_bytes()).unwrap();
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                                _ => {}
                                            }
                                        }

                                        if let Ok(()) = rx.try_recv() {
                                            break;
                                        }
                                    }
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
                            if fields.playerid == client.game.players[client.localplayerid as usize].playerid
                            {
                                unsafe {
                                    enet_peer_disconnect_now(client.peer, 0);
                                    enet_deinitialize();
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
