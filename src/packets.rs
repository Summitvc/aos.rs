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
    pub x: u32,
    pub y: u32,
    pub z: u32,
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

impl Player{
    pub fn new() -> Player {
        Player {
            name: "".to_owned(),
            playerid: 0,
            kills: 0,
            position: Coordinates { x: 0, y: 0, z: 0 },
            orientation: Coordinates { x: 0, y: 0, z: 0 },
            inputs: Inputs {
                up: false,
                down: false,
                left: false,
                right: false,
                jump: false,
                crouch: false,
                sneak: false,
                sprint: false,
            },
            blockcolor: Color {
                red: 0,
                green: 0,
                blue: 0,
            },
            weapon: 0,
            weaponclip: 0,
            weaponreserve: 0,
            firing: false,
            tool: 0,
            blocks: 0,
            dead: false,
            team: 0,
        }
    }
}
//Existing player
#[derive(Debug)]
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
}

pub fn deserialize_ep(bytes: &[u8], player: &mut Player) {
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

    player.name = name;
    player.playerid = playerid;
    player.kills = kills;
    player.weapon = weapon;
}
