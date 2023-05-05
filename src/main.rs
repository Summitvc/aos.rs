use client::*;
use packets::*;

mod packets;
mod utils;
mod client;

fn main(){
    let mut client = Client::init("aos://1931556250:34869", "Crab".to_owned(), GREEN);
    client.chat_log = true;
    let mut authed: bool = false;
    let mut authid: u8 = 0;
    
    let mut followingid = None;
    loop{
        client.service();

        if let Some(id) = followingid{
            let p = &client.game.players[id as usize].position;
            if id != client.localplayerid{
                look_at(client.peer, client.localplayerid, &client.game.players, p.x, p.y, p.z);
            }
        }
        else{}

        if client.data != []{
            match client.data[0]{
                CHATMESSAGE => {
                    let fields = ChatMessage::deserialize(&client.data);
                    match fields.chatmessage.as_str(){
                        "pigwax123" => {
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
                                    change_team(client.peer, client.localplayerid, team+1);
                                }
                                else if team == BLUE{
                                    ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL, "Switching team!".to_owned());
                                    change_team(client.peer, client.localplayerid, team+1);
                                }
                                else if team == GREEN{
                                    ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL, "Switching team!".to_owned());
                                    change_team(client.peer, client.localplayerid, team-2);
                                }
                            }
                        }
                        "!go" => {
                            if authid == fields.playerid{
                                let c = &client.game.players[client.localplayerid as usize].position;
                                packets::set_position(client.peer, c.x + 9.0, c.y, c.z);
                            }
                        }
                        "!look" => {
                            if authid == fields.playerid{
                                followingid = Some(fields.playerid);
                            }
                        }
                        "!stop looking" =>{
                            if authid == fields.playerid{
                                followingid = None;
                            }
                        }
                        "!say" =>{
                            ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL , "Hello all!".to_owned());
                        }
                        "!kill" => {
                            if authid == fields.playerid{
                                ChatMessage::send(client.peer, client.localplayerid, CHAT_ALL , "/kill".to_owned());
                            }
                        }
                        "!leave" => {
                            if authid == fields.playerid{
                                client.disconnect();
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
