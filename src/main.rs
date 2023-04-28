use client::{Client};

mod packets;
mod utils;
mod client;


fn main(){
    let mut  client = Client::init("aos://1931556250:34869", "Crab".to_owned());

    loop{
        client.service();
        if client.data != []{
            match client.data[0]{
                packets::CHATMESSAGE => {
                    // if client.data[1] < 32{
                    //     println!("{}: {}", client.game.players[client.data[1] as usize].name, packets::ChatMessage::deserialize(&client.data))
                    // }
                }

                _ => {}
            }
        }
    }
}