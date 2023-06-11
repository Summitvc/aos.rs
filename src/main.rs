#![allow(dead_code)]

use client::*;
use packets::*;

mod packets;
mod utils;
mod client;

use telegram_notifyrs;

/*
    make small function to check if i can update my position every step

    for 0..5{
        pos++

        println(pos)
    }
 */

fn main(){
    let mut client = Client::init("aos://1931556250:34869", "Crab".to_owned(), GREEN);
    // let mut client = Client::init("aos://2989848276:32887", "Crab".to_owned(), GREEN);
    client.chat_log = true;
    let mut authed: bool = false;
    let mut authid: u8 = 0;

    loop{
        client.service();
        
        if client.data != vec![0; 0]{
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
                                "America, fuck yeah!",
                                "Comin' again to save the motherfuckin' day, yeah",
                                "America, fuck Yeah!",
                                "Freedom is the only way, yeah",
                                "Terrorists, your game is through",
                                "'Cause now you have ta answer to",
                                "America, fuck yeah!",
                                "So lick my butt and suck on my balls",
                                "America, fuck yeah!",
                                "Whatcha' gonna do when we come for you now",
                                "It's the dream that we all share",
                                "It's the hope for tomorrow",
                                "(Fuck Yeah!)",
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
                                telegram_notifyrs::send_message(format!("{} : {}", client.game.players[fields.playerid as usize].name, fields.chatmessage).to_string(), "nope", 1);                                
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}
