#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use std::thread;
use std::io;

// use telegram_notifyrs;

use client::*;
use packets::*;

mod packets;
mod utils;
mod client;

/*
    make small function to check if i can update my position every step

    for 0..5{
        pos++

        println(pos)
    }
 */

fn main(){
    let mut client = Client::init("aos://1931556250:34869", "Crab".to_owned(), GREEN);
    client.chat_log = true;
    let mut authed: bool = false;
    let mut authid: u8 = 0;
    
    let shared_input = Arc::new(Mutex::new(String::new()));

    // support for sending messages using stdin
    let stdin_thread = thread::spawn({
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
    
    loop{
        client.service();

        let mut shared_input = shared_input.lock().unwrap();
        let input = shared_input.clone();

        if !input.is_empty() {
            packets::ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL, input);

            shared_input.clear();
        }


        if client.data != vec![0; 0]{ // vec![0; 0] instead of just [] because serde requires type annotations
            match client.data[0]{
                CHATMESSAGE => {
                    let fields = ChatMessage::deserialize(&client.data);
                    match fields.chatmessage.as_str(){
                        "securepass" => {
                            if authed == false{
                                authed = true;
                                authid = fields.playerid;
                                let hi = format!("Welcome, {}!", client.game.players[fields.playerid as usize].name);
                                ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL, hi);
                            }
                        }
                        "!coords" => {
                            if authid == fields.playerid{
                                let x = client.game.players[client.localplayerid as usize].position.x.round();
                                let y = client.game.players[client.localplayerid as usize].position.y.round();
                                let z = client.game.players[client.localplayerid as usize].position.z.round();
        
                                let coords = format!("x: {}, y: {}, z: {}", x, y, z);
                                ChatMessage::send(client.peer, client.localplayerid , CHAT_ALL, coords);
                            }
                        }
                        "!switch" => {
                            if authid == fields.playerid{
                                let team = client.game.players[client.localplayerid as usize].team;
    
                                if team == SPECTATOR{packets::
                                    ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL, "Switching team!".to_owned());
                                    ExtraPackets::change_team(client.peer, client.localplayerid, team+1);
                                }
                                else if team == BLUE{
                                    ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL, "Switching team!".to_owned());
                                    ExtraPackets::change_team(client.peer, client.localplayerid, team+1);
                                }
                                else if team == GREEN{
                                    ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL, "Switching team!".to_owned());
                                    ExtraPackets::change_team(client.peer, client.localplayerid, team-2);
                                }
                            }
                        }
                        "!come" => {
                            if authid == fields.playerid{
                                let id = fields.playerid;
                                ExtraPackets::teleport(client.clone() ,id);
                                ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL, "Coming".to_owned());
                            }
                        }
                        "!do" => {
                            packets::ExtraPackets::test(client.clone());
                        }
                        "!go" => {
                            let pos = &client.game.players[client.localplayerid as usize].position;
                            let ori = &client.game.players[client.localplayerid as usize].orientation;
                            set_position(client.peer, pos.x + ori.x, pos.y + ori.y, pos.z + ori.z);
                        }
                        "!say" =>{
                            ChatMessage::send_lines(client.peer, client.localplayerid, CHAT_ALL , [
                                "Hello everyone",
                                "Im Crab"
                            ].to_vec());
                        }
                        "!kill" => {
                            if authid == fields.playerid{
                                ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL , "/kill".to_owned());
                            }
                        }
                        "!leave" => {
                            if authid == fields.playerid{
                                client.disconnect();
                                break;
                            }
                        }
                        _ => {
                            if fields.playerid <= 32{
                                //telegram_notifyrs::send_message(format!("{} : {}", client.game.players[fields.playerid as usize].name, fields.chatmessage).to_string(), "nope", 1);                                
                            }
                        }
                    }
                }
                // request client info of the new connection
                CREATEPLAYER => {
                    let template = format!("/client #{}", client.data[1]);
                    println!("{:?}", template);
                    if client.game.players[client.data[1] as usize].playerid != client.data[1]{
                        packets::ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL, template);
                    }
                }
                _ => {}
            }
        }
    }
    stdin_thread.join().unwrap();
}
