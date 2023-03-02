mod net;
mod note;
mod typ;

use anyhow::Result;
use note::Note;
use typ::TornadoUtil;

pub struct Tornado {
    note_list: Vec<Note>,
    util: TornadoUtil,
}

impl Default for Tornado {
    fn default() -> Self {
        Self {
            note_list: vec![],
            util: TornadoUtil::new(),
        }
    }
}

impl Tornado {
    pub async fn new(list: Vec<&str>) -> Result<Self> {
        let s = Self::default();
        s.util.init().await;
        s.parse_note(list)
    }

    pub async fn prepare_check_block(&self) -> Result<bool> {
        todo!()
    }

    fn parse_note(mut self, list: Vec<&str>) -> Result<Self> {
        self.note_list = list
            .iter()
            .map(|note| Note::new(note, &self.util).unwrap())
            .collect();

        Ok(self)
    }
}
