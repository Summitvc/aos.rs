use client::{Client};

mod packets;
mod utils;
mod client;


fn main(){
    let mut  client = Client::init("aos://1931556250:34869", "Kid".to_owned());

    loop{
        client.service();
        if client.data.len() > 0{
            match client.data[0] {
                _ => {}
            }
        }
    }

}  