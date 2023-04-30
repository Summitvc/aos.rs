use client::{Client};
use enet_sys::enet_peer_disconnect;

mod packets;
mod utils;
mod client;


fn main(){
    let mut client = Client::init("aos://1931556250:34869", "Crab".to_owned());
    client.chat_log = true;

    loop{
        client.service();
        if client.data != []{
            match client.data[0]{
                packets::CHATMESSAGE => {
                    let fields = packets::ChatMessage::deserialize(&client.data);
                    match fields.chatmessage.as_str(){
                        "!coords" => {
                            let x = client.game.players[client.statedata.localplayerid as usize].position.x.round();
                            let y = client.game.players[client.statedata.localplayerid as usize].position.y.round();
                            let z = client.game.players[client.statedata.localplayerid as usize].position.z.round();
    
                            let coords = format!("x: {}, y: {}, z: {}", x, y, z);
                            packets::ChatMessage::send(client.peer, client.statedata.localplayerid , coords);
                        }
                        "!test" => {
                            packets::ChatMessage::send(client.peer, client.statedata.localplayerid , "﷽".to_owned());
                        }
                        "!leave" => {
                            client.disconnect();
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}