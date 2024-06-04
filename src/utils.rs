use std::{fmt::Error, sync::mpsc, thread, time::Duration};

use crate::Client;

pub fn ip(address: &str) -> Result<Vec<String>, Error> {
    if !address.starts_with("aos://") {
        return Ok(address.split(":").map(String::from).collect());
    }

    let address = &address["aos://".len()..];
    let mut addr_split = address.split(":").map(String::from).collect::<Vec<_>>();

    if addr_split[0].split(".").nth(1).is_some() {
        return Ok(addr_split);
    }

    let ip: u32 = match addr_split[0].parse() {
        Ok(ip) => ip,
        Err(_) => {
            println!("Wrong ip format");
            return Err(Error);
        }
    };

    addr_split[0] = format!(
        "{}.{}.{}.{}",
        ip & 255,
        (ip >> 8) & 255,
        (ip >> 16) & 255,
        (ip >> 24) & 255
    );

    Ok(addr_split)
}
