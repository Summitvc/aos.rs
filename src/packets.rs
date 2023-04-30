use std::ffi::c_void;

use enet_sys::*;

pub const WORLDUPDATE: u8 = 2;
pub const EXISTINGPLAYER: u8 = 9;
pub const CREATEPLAYER: u8 = 12;
pub const STATEDATA: u8 = 15;
pub const KILLACTION: u8 = 16;
pub const CHATMESSAGE: u8 = 17;
pub const MAPSTART: u8 = 18;
pub const MAPCHUNK: u8 = 19;
pub const MAPCACHED: u8 = 31;

#[derive(Clone, Debug, Default)]
pub struct Coordinates {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Debug, Default)]
pub struct Inputs {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub crouch: bool,
    pub sneak: bool,
    pub sprint: bool,
}
#[derive(Clone, Debug, Default)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

#[derive(Clone, Debug, Default)]
pub struct Player {
    pub name: String,
    pub playerid: u8,
    pub kills: u32,
    pub position: Coordinates,
    pub orientation: Coordinates,
    pub inputs: Inputs,
    pub blockcolor: Color,
    pub weapon: u8,
    pub weaponclip: u8,
    pub weaponreserve: u8,
    pub firing: bool,
    pub tool: u8,
    pub blocks: u8,
    pub dead: bool,
    pub team: i8,
}

#[derive(Debug, Default)]
pub struct StateData{
    pub localplayerid: u8,
    pub fog_b: u8,
    pub fog_g: u8,
    pub fog_r: u8,
    pub team1_b: u8,
    pub team1_g: u8,
    pub team1_r: u8,
    pub team2_b: u8,
    pub team2_g: u8,
    pub team2_r: u8,
    pub team1name: String,
    pub team2name: String,
    pub gamemode: u8,
}

#[derive(Debug, Default)]
pub struct ExistingPlayer {
    pub playerid: u8,
    pub team: i8,
    pub weapon: u8,
    pub helditem: u8,
    pub kills: u32,
    pub blue: u8,
    pub green: u8,
    pub red: u8,
    pub name: String,
}
#[derive(Default)]
pub struct ChatMessage{
    pub playerid: u8,
    pub chattype: u8,
    pub chatmessage: String,
}

impl StateData{
    pub fn deserialize_join(&mut self, bytes: &[u8], name: String, peer: *mut _ENetPeer, players: &mut Vec<Player>){
        let mut buf:Vec<u8> = bytes.to_vec();

        buf.remove(0);

        self.localplayerid = buf[0];
        self.fog_b = buf[1];
        self.fog_g = buf[2];
        self.fog_r = buf[3];
        self.team1_b = buf[4];
        self.team1_g = buf[5];
        self.team1_r = buf[6];
        self.team2_b = buf[7];
        self.team2_g = buf[8];
        self.team2_r = buf[9];
        self.team1name = String::from_utf8(buf[10..20].try_into().unwrap()).unwrap();
        self.team2name = String::from_utf8(buf[20..30].try_into().unwrap()).unwrap();   

        players[self.localplayerid as usize].playerid = self.localplayerid;
        players[self.localplayerid as usize].name = name.clone();

        let mut exps: Vec<u8> = ExistingPlayer::serialize(&Default::default(), name.clone());
        let exps_ptr: *const c_void = exps.as_mut_ptr() as *mut c_void;

        unsafe{
            let new_packet = enet_packet_create(
                exps_ptr,
                exps.len() as u64,
                _ENetPacketFlag_ENET_PACKET_FLAG_RELIABLE
            );
            
            enet_peer_send(peer, 0, new_packet);   
        }
    }
}

impl ChatMessage{
    pub fn send(peer: *mut _ENetPeer, localplayerid: u8, message: String){
        let mut buf: Vec<u8> = Vec::new();

        buf.push(CHATMESSAGE);
        buf.push(localplayerid);
        buf.push(0); // - global
        buf.append(&mut message.as_bytes().to_vec());
        buf.push(0);

        let message_ptr: *const c_void = buf.as_mut_ptr() as *mut c_void;
        unsafe{
            let packet = enet_packet_create(message_ptr, buf.len() as u64, _ENetPacketFlag_ENET_PACKET_FLAG_RELIABLE);
            enet_peer_send(peer, 0, packet);
        }

    }
    pub fn deserialize(bytes: &[u8]) -> ChatMessage{
        let buf = bytes[3..bytes.len()-1].to_vec();

        ChatMessage { playerid: bytes[1], chattype: bytes[2], chatmessage: String::from_utf8_lossy(&buf).to_string()}
    }
}

impl ExistingPlayer {
    pub fn serialize(&self, name: String) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.push(EXISTINGPLAYER);
        buf.push(self.playerid as u8);
        buf.push(self.team as u8);
        buf.push(self.weapon as u8);
        buf.push(self.helditem as u8);
        buf.extend(self.kills.to_le_bytes());
        buf.push(self.blue as u8);
        buf.push(self.green as u8);
        buf.push(self.red as u8);
        buf.extend(name.as_bytes());

        return buf;
    }
    pub fn deserialize(bytes: &[u8], players: &mut Vec<Player>) {
        //shifted by 1 index to right due to id
        let playerid = bytes[1];
        // let team = bytes[2] as i8;
        let weapon = bytes[3];
        // let helditem = bytes[4];
        let kills = u32::from_le_bytes(bytes[5..9].try_into().unwrap());
        // let blue = bytes[9];
        // let green = bytes[10];
        // let red = bytes[11];
        let name: String = String::from_utf8_lossy(&bytes[12..]).into_owned();

        players[playerid as usize].name = name;
        players[playerid as usize].playerid = playerid;
        players[playerid as usize].kills = kills;
        players[playerid as usize].weapon = weapon;
    }
}

#[derive(Default)]
pub struct WorldUpdate {
    // posx: f32,
    // posy: f32,
    // posz: f32,
    // orix: f32,
    // oriy: f32,
    // oriz: f32,
}

impl WorldUpdate {
    pub fn deserialize(bytes: &[u8], players: &mut Vec<Player>) {
        let mut id = 0;
        let mut index = 1;
        let mut buf: Vec<u8> = Vec::new();

        if let Some((_, remaining)) = bytes.split_first() {
            for item in remaining {
                buf.push(*item);
                if buf.len() == 24 {
                    let chop = buf.chunks_exact(4);
                    for i in chop {
                        let vec = i.to_vec();
                        let mut iter = vec.into_iter();
                        let f: [u8; 4] = {
                            [
                                iter.next().unwrap_or(0),
                                iter.next().unwrap_or(0),
                                iter.next().unwrap_or(0),
                                iter.next().unwrap_or(0),
                            ]
                        };

                        match index {
                            1 => players[id].position.x = f32::from_le_bytes(f),
                            2 => players[id].position.y = f32::from_le_bytes(f),
                            3 => players[id].position.z = f32::from_le_bytes(f),
                            4 => players[id].orientation.x = f32::from_le_bytes(f),
                            5 => players[id].orientation.y = f32::from_le_bytes(f),
                            6 => players[id].orientation.z = f32::from_le_bytes(f),
                            _ => println!("error at matching index at worldupdate::deserialize()"),
                        }
                        index += 1;
                    }
                    index = 1;
                    buf.clear();
                    id += 1;
                }
            }
        }
    }
}
