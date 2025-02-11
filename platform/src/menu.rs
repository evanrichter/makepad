use {
    crate::{
        makepad_live_id::LiveId,
        event::KeyCode
    },
};


#[derive(Clone, Copy, Default)]
pub struct CxCommandSetting {
    pub shift: bool,
    pub key_code: KeyCode,
    pub enabled: bool
}

// Command

#[derive(Clone, Debug, Default, Eq, Hash, Copy, PartialEq)]
pub struct Command(pub LiveId);
impl From<LiveId> for Command {
    fn from(live_id: LiveId) -> Command {Command(live_id)}
}

impl Command{
    //pub fn from_id(id:LiveId)->Self{Self(id.0)}
    /*
    pub fn set_enabled(&self, cx:&mut Cx, enabled:bool)->Self{
        let mut s = if let Some(s) = cx.command_settings.get(self){*s}else{CxCommandSetting::default()};
        s.enabled = enabled;
        cx.command_settings.insert(*self, s);
        *self
    }

    pub fn set_key(&self, cx:&mut Cx, key_code:KeyCode)->Self{
        let mut s = if let Some(s) = cx.command_settings.get(self){*s}else{CxCommandSetting::default()};
        s.shift = false;
        s.key_code = key_code;
        cx.command_settings.insert(*self, s);
        *self
    }
    
    pub fn set_key_shift(&self, cx:&mut Cx, key_code:KeyCode)->Self{
        let mut s = if let Some(s) = cx.command_settings.get(self){*s}else{CxCommandSetting::default()};
        s.shift = true;
        s.key_code = key_code;
        cx.command_settings.insert(*self, s);
        *self
    }*/
}

#[derive(PartialEq, Clone)]
pub enum Menu {
    Main {items:Vec<Menu>},
    Item {name: String, command:Command},
    Sub {name: String, items: Vec<Menu>},
    Line
}

impl Menu {
    pub fn main(items: Vec<Menu>)->Menu{
        Menu::Main{items:items}
    }
    
    pub fn sub(name: &str, items: Vec<Menu>) -> Menu {
        Menu::Sub {
            name: name.to_string(),
            items: items
        }
    }
    
    pub fn line() -> Menu {
        Menu::Line
    }
    
    pub fn item(name: &str, command: Command) -> Menu {
        Menu::Item {
            name: name.to_string(),
            command: command
        }
    }
}