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

//Existing player
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
pub struct ChatMessage{
    pub playerid: u8,
    pub chattype: u8,
    pub chatmessage: String,
}

impl ChatMessage{
    // pub fn serialize(&self, message: String) -> Vec<u8>{
    //     let mut buf: Vec<u8> = Vec::new();
        
    // }
    pub fn deserialize(bytes: &[u8]) -> String{
        let mut buf = bytes.to_vec();

        buf.remove(0);
        buf.remove(1);

        return String::from_utf8(buf).unwrap();
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
