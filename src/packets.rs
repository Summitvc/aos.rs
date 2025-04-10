use std::thread;
use std::time::{self, Duration};

use codepage_437::{CP437_WINGDINGS, FromCp437};
use enet_sys::*;

use bit_vec::BitVec;

use std::sync::mpsc;

use crate::client::Client;

pub const WORLDUPDATE: u8 = 2;
pub const EXISTINGPLAYER: u8 = 9;
pub const CREATEPLAYER: u8 = 12;
pub const STATEDATA: u8 = 15;
pub const KILLACTION: u8 = 16;
pub const CHATMESSAGE: u8 = 17;
pub const PLAYERLEFT: u8 = 20;
pub const GRENADEPACKET: u8 = 6;
pub const HANDSHAKE_INIT: u8 = 31;
pub const HANDSHAKE_RETURN: u8 = 32;
pub const VERSION_REQ: u8 = 33;
pub const VERSION_RESP: u8 = 34;
pub const MAPSTART: u8 = 18;
pub const MAPCHUNK: u8 = 19;
pub const MAPCACHED: u8 = 31;

pub const CHAT_ALL: u8 = 0;
pub const CHAT_TEAM: u8 = 1;
pub const CHAT_SYSTEM: u8 = 2;

pub const X_SIZE: i32 = 512;
pub const Y_SIZE: i32 = 512;
pub const Z_SIZE: i32 = 64;

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
    pub grenades: u8,
    pub firing: bool,
    pub tool: u8,
    pub blocks: u8,
    pub dead: bool,
    pub team: i8,
    pub connected: bool,
}

#[derive(Debug, Default, Clone)]
pub struct StateData {
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
pub struct ChatMessage {
    pub playerid: u8,
    pub chattype: u8,
    pub chatmessage: String,
}
#[derive(Default, Debug)]
pub struct KillAction {
    pub playerid: u8,
    pub killerid: u8,
    pub killtype: u8,
    pub respawntime: u8,
}
#[derive(Default, Debug)]
pub struct CreatePlayer {
    pub playerid: u8,
    pub weapon: u8,
    pub team: i8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub name: String,
}
pub struct VersionInfo {
    pub client: i8,
    pub version_major: u8,
    pub version_minor: u8,
    pub version_revision: u8,
    pub operating_system_info: String,
}

#[derive(Default, Clone)]
pub struct WorldMap {
    pub data: Vec<u8>,
    pub blocks: BitVec,
    pub colors: Vec<Vec<Vec<Color>>>,
}

pub struct ExtraPackets {}

impl CreatePlayer {
    pub fn deserialize(mut self, players: &mut Vec<Player>, data: &[u8]) {
        let mut filter = data.to_vec();
        self.playerid = data[1];
        self.weapon = data[2];
        self.team = data[3] as i8;
        self.x = f32::from_le_bytes(data[4..8].try_into().unwrap());
        self.y = f32::from_le_bytes(data[8..12].try_into().unwrap());
        self.z = f32::from_le_bytes(data[12..16].try_into().unwrap());

        //utf
        if filter[16] == 255 {
            filter[16] = 0
        }

        self.name = String::from_utf8_lossy(&filter[16..data.len() - 1]).to_string();

        //check if player joined or respawned
        if players[self.playerid as usize].connected != true {
            players[self.playerid as usize].name = self.name;
            players[self.playerid as usize].connected = true;
            players[self.playerid as usize].dead = false;
        }
        players[self.playerid as usize].weapon = self.weapon;
        players[self.playerid as usize].team = self.team;
        players[self.playerid as usize].position.x = self.x;
        players[self.playerid as usize].position.x = self.y;
        players[self.playerid as usize].position.x = self.z;
        players[self.playerid as usize].grenades = 3;
        players[self.playerid as usize].dead = false;
    }
}

pub fn throw_grenade(
    peer: *mut _ENetPeer,
    localplayerid: u8,
    players: &mut Vec<Player>,
    fuselength: f32,
    xp: f32,
    yp: f32,
    zp: f32,
    xv: f32,
    yv: f32,
    zv: f32,
) {
    let mut buf = Vec::new();

    buf.push(GRENADEPACKET);
    buf.push(localplayerid);
    buf.extend_from_slice(&fuselength.to_le_bytes());
    buf.extend_from_slice(&xp.to_le_bytes());
    buf.extend_from_slice(&yp.to_le_bytes());
    buf.extend_from_slice(&zp.to_le_bytes());
    buf.extend_from_slice(&xv.to_le_bytes());
    buf.extend_from_slice(&yv.to_le_bytes());
    buf.extend_from_slice(&zv.to_le_bytes());
    if players[localplayerid as usize].grenades > 0 {
        players[localplayerid as usize].grenades -= 1;
        send(peer, buf);
    }
}

pub fn set_position(peer: *mut _ENetPeer, x: f32, y: f32, z: f32) {
    let mut buf: Vec<u8> = Vec::new();

    buf.push(0);
    buf.extend_from_slice(&x.to_le_bytes());
    buf.extend_from_slice(&y.to_le_bytes());
    buf.extend_from_slice(&z.to_le_bytes());

    send(peer, buf);
}

pub fn set_orientation(peer: *mut _ENetPeer, x: f32, y: f32, z: f32) {
    let mut buf: Vec<u8> = Vec::new();

    buf.push(1);
    buf.extend_from_slice(&x.to_le_bytes());
    buf.extend_from_slice(&y.to_le_bytes());
    buf.extend_from_slice(&z.to_le_bytes());

    send(peer, buf);
}

impl ExtraPackets {
    pub fn test(mut client: Client) {
        for i in 0..3 {
            println!(
                "{:?}",
                client.game.players[client.localplayerid as usize].position
            );
            let pos = &client.game.players[client.localplayerid as usize].position;
            set_position(client.peer, pos.x + i as f32, pos.y, pos.z);
            unsafe {
                enet_host_flush(client.client);
            }
            client.service();
            thread::sleep(time::Duration::from_secs(1));
        }
    }
    pub fn look_at(
        peer: *mut _ENetPeer,
        localplayerid: u8,
        players: &Vec<Player>,
        x: f32,
        y: f32,
        z: f32,
    ) {
        let bot_pos = &players[localplayerid as usize].position;

        let x_diff = x - bot_pos.x;
        let y_diff = y - bot_pos.y;
        let z_diff = z - bot_pos.z;

        let mag = (x_diff * x_diff + y_diff * y_diff + z_diff * z_diff).sqrt();
        let (x_norm, y_norm, z_norm) = if mag != 0.0 {
            (x_diff / mag, y_diff / mag, z_diff / mag)
        } else {
            (0.0, 0.0, 0.0) // Avoid division by zero
        };

        set_orientation(peer, x_norm, y_norm, z_norm);
    }

    pub fn change_team(peer: *mut _ENetPeer, id: u8, team: i8) {
        let buf: Vec<u8> = vec![29, id, team as u8];

        send(peer, buf);
    }
    pub fn teleport(client: Client, id: u8) {
        let player_pos = &client.game.players[id as usize].position;
        let bot_pos = &client.game.players[client.localplayerid as usize].position;
        let ori = &client.game.players[client.localplayerid as usize].orientation;
        ExtraPackets::look_at(
            client.peer,
            client.localplayerid,
            &client.game.players,
            player_pos.x,
            player_pos.y,
            player_pos.z,
        );
        unsafe {
            enet_host_flush(client.client);
        }
        thread::sleep(Duration::from_millis(2000));

        client.clone().service();

        let xdiff = player_pos.x - bot_pos.x;
        let ydiff = player_pos.y - bot_pos.y;
        let zdiff = player_pos.z - bot_pos.z;

        let length = (xdiff * xdiff + ydiff * ydiff + zdiff * zdiff)
            .sqrt()
            .round();

        println!("length {}", length);

        let step = 1;
        let steps: u32 = length as u32 / step;

        println!("steps {}", steps);

        for k in 0..steps {
            let bot_pos2 = &client.game.players[client.localplayerid as usize].position;
            set_position(
                client.peer,
                (bot_pos2.x + ori.x) + k as f32,
                (bot_pos2.y + ori.y) + k as f32,
                (bot_pos2.z + ori.z) + k as f32,
            );
            println!("{:?}", bot_pos2);
            client.clone().service();
            thread::sleep(Duration::from_millis(1000));
        }
    }
}

impl KillAction {
    pub fn deserialize(mut self, players: &mut Vec<Player>, data: &[u8]) {
        self.playerid = data[1];
        self.killerid = data[2];
        self.killtype = data[3];
        self.respawntime = data[4];

        players[self.playerid as usize].dead = true;
        players[self.killerid as usize].kills += 1;
    }
}

pub fn send(peer: *mut _ENetPeer, data: Vec<u8>) {
    let buf_ptr: *const _ = data.as_ptr() as *mut _;

    unsafe {
        let new_packet = enet_packet_create(
            buf_ptr,
            data.len() as usize,
            _ENetPacketFlag_ENET_PACKET_FLAG_RELIABLE,
        );

        enet_peer_send(peer, 0, new_packet);
    }
}

pub fn join(peer: *mut _ENetPeer, name: String, team: i8) {
    let exps: Vec<u8> = ExistingPlayer::serialize(&Default::default(), name, team);

    send(peer, exps);
}

impl StateData {
    pub fn deserialize(&mut self, players: &mut Vec<Player>, localplayerid: &mut u8, data: &[u8]) {
        let mut buf: Vec<u8> = data.to_vec();

        buf.remove(0);

        *localplayerid = buf[0];
        players[buf[0] as usize].playerid = buf[0];
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
    }
}

impl ChatMessage {
    pub fn send(peer: *mut _ENetPeer, localplayerid: u8, chattype: u8, message: String) {
        let mut buf: Vec<u8> = Vec::new();

        buf.push(CHATMESSAGE);
        buf.push(localplayerid);
        buf.push(chattype);

        // 255 to indicate utf-8
        buf.push(255);
        buf.append(&mut message.as_bytes().to_vec());
        buf.push(0);

        send(peer, buf);
    }
    pub fn send_lines(
        client: &mut Client,
        peer: *mut _ENetPeer,
        localplayerid: u8,
        chattype: u8,
        lines: Vec<&str>,
    ) {
        let mut g = true; // toggle sending
        let mut k = 0; // counter
        let mut a = lines.iter();

        let (tx, rx) = mpsc::channel();
        loop {
            if g == true {
                ChatMessage::send(peer, localplayerid, chattype, a.next().unwrap().to_string());

                let tx_clone = tx.clone();

                thread::spawn(move || {
                    thread::sleep(Duration::from_secs(2));
                    tx_clone.send(()).unwrap();
                });
            }
            g = false;
            client.service();

            if let Ok(()) = rx.try_recv() {
                k += 1;
                g = true;
            }
            if k == lines.len() {
                break;
            }
        }
    }
    pub fn deserialize(data: &[u8]) -> ChatMessage {
        let mut buf = data[3..data.len() - 1].to_vec();

        if buf.len() > 0 && buf[0] == 255 {
            buf[0] = 0;
            ChatMessage {
                playerid: data[1],
                chattype: data[2],
                chatmessage: String::from_utf8_lossy(&buf).to_string(),
            }
        } else {
            ChatMessage {
                playerid: data[1],
                chattype: data[2],
                chatmessage: String::from_cp437(buf, &CP437_WINGDINGS),
            }
        }
    }
}

impl ExistingPlayer {
    pub fn serialize(&self, name: String, team: i8) -> Vec<u8> {
        let mut buf = Vec::new();

        buf.push(EXISTINGPLAYER);
        buf.push(self.playerid);
        buf.push(team as u8);
        buf.push(self.weapon);
        buf.push(self.helditem);
        buf.extend(self.kills.to_le_bytes());
        buf.push(self.blue);
        buf.push(self.green);
        buf.push(self.red);

        //255 utf indication
        buf.push(255);
        buf.extend(name.as_bytes());

        return buf;
    }
    pub fn deserialize(data: &[u8], players: &mut Vec<Player>) {
        let mut filter = data.to_vec();
        //shifted by 1 index to right due to id
        let playerid = data[1];
        // let team = bytes[2] as i8;
        let weapon = data[3];
        // let helditem = bytes[4];
        let kills = u32::from_le_bytes(data[5..9].try_into().unwrap());
        // let blue = bytes[9];
        // let green = bytes[10];
        // let red = bytes[11];

        // FITER 255 BYTE OUT
        if filter[12] == 255 {
            filter[12] = 0
        }
        let name: String = String::from_utf8_lossy(&filter[12..]).into_owned();

        players[playerid as usize].name = name;
        players[playerid as usize].playerid = playerid;
        players[playerid as usize].kills = kills;
        players[playerid as usize].weapon = weapon;
        players[playerid as usize].connected = true;
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

impl WorldMap {
    pub fn get_top_block(x: i32, y: i32, map: &mut WorldMap) -> u8 {
        for i in 0..Z_SIZE {
            let block = map
                .blocks
                .get((x + y * X_SIZE + i * X_SIZE * Y_SIZE) as usize)
                .unwrap();
            if block == true {
                return i as u8;
            }
        }
        return 63;
    }

    pub fn setgeom(x: i32, y: i32, z: i32, t: bool, map: &mut WorldMap) {
        assert!(z >= 0 && z < Z_SIZE);
        map.blocks
            .set((x + y * X_SIZE + z * X_SIZE * Y_SIZE) as usize, t);
    }

    pub fn setcolor(x: i32, y: i32, z: i32, c: Color, map: &mut WorldMap) {
        assert!(z >= 0 && z < Z_SIZE);
        map.colors[x as usize][y as usize][z as usize] = c;
    }

    pub fn deserialize(map: &mut WorldMap) {
        let base: Vec<u8> = map.data.clone();

        let mut offset: i32 = 0;

        for y in 0..Y_SIZE {
            for x in 0..X_SIZE {
                for z in 0..Z_SIZE {
                    WorldMap::setgeom(x, y, z, true, map);
                }
                let mut z = 0;
                loop {
                    let number_4byte_chunks: i32 = base[offset as usize].into();
                    let top_color_start: i32 = base[offset as usize + 1].into();
                    let top_color_end: i32 = base[offset as usize + 2].into();

                    for i in z..top_color_start {
                        WorldMap::setgeom(x, y, i, false, map);
                    }

                    let mut color_offset: usize = offset as usize + 4;
                    for i in top_color_start..=top_color_end {
                        let c: Color = Color {
                            red: base[color_offset + 2],
                            green: base[color_offset + 1],
                            blue: base[color_offset + 0],
                        };

                        WorldMap::setcolor(x, y, i, c, map);
                        color_offset += 4;
                    }

                    let len_bottom = top_color_end - top_color_start + 1;

                    if number_4byte_chunks == 0 {
                        offset += 4 * (len_bottom + 1);
                        break;
                    }

                    let len_top = (number_4byte_chunks - 1) - len_bottom;
                    offset += base[offset as usize] as i32 * 4;

                    let bottom_color_end: i32 = base[offset as usize + 3] as i32;
                    let bottom_color_start = bottom_color_end - len_top;

                    for i in bottom_color_start..bottom_color_end {
                        let c: Color = Color {
                            red: base[color_offset + 2],
                            green: base[color_offset + 1],
                            blue: base[color_offset + 0],
                        };

                        WorldMap::setcolor(x, y, i, c, map);
                        color_offset += 4;
                    }
                    z = bottom_color_end;
                }
            }
        }
    }
}

impl WorldUpdate {
    pub fn deserialize(data: &[u8], players: &mut Vec<Player>) {
        let mut id = 0;
        let mut index = 1;
        let mut buf: Vec<u8> = Vec::new();

        if let Some((_, remaining)) = data.split_first() {
            for item in remaining {
                buf.push(*item);
                if buf.len() == 24 {
                    let chop = buf.chunks_exact(4);
                    for i in chop {
                        let vec = i.to_vec();
                        let mut iter = vec.into_iter();
                        let mut f: [u8; 4] = Default::default();
                        for i in 0..4 {
                            f[i] = iter.next().unwrap_or(0);
                        }
                        match index {
                            1 => players[id].position.x = f32::from_le_bytes(f),
                            2 => players[id].position.y = f32::from_le_bytes(f),
                            3 => players[id].position.z = f32::from_le_bytes(f),
                            4 => players[id].orientation.x = f32::from_le_bytes(f),
                            5 => players[id].orientation.y = f32::from_le_bytes(f),
                            6 => players[id].orientation.z = f32::from_le_bytes(f),
                            _ => panic!("error at matching index at worldupdate::deserialize()"),
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
