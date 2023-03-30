use serde_derive::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct ExistingPlayer{
    pub player_id:   u8,
    pub team:       u8,
    pub weapon:     u8,
    pub held_item:   u8,
    pub kills: 	    u8,
    pub blue: 	    u8,
    pub green: 	    u8,
    pub red: 	    u8, 
    pub name: 	    String
}