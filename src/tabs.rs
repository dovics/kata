mod broker;
mod group;
mod topic;
pub use broker::BrokerTab;
pub use topic::TopicTab;

use crate::app::{App, Mode};
use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use strum::{Display, EnumIter, FromRepr};

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, FromRepr, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Topic,
    Group,
    Broker,
}

impl Tab {
    pub fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    pub fn prev(self) -> Self {
        let current_index = self as usize;
        let prev_index = current_index.saturating_sub(1);
        Self::from_repr(prev_index).unwrap_or(self)
    }

    pub fn title(self) -> String {
        match self {
            Self::Topic => String::from("Topic"),
            Self::Group => String::from("Group"),
            Self::Broker => String::from("Broker"),
        }
    }
}

impl App {
    pub fn handle_tab_select(&mut self, key: KeyEvent) -> Result<Mode> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return Ok(Mode::Quit),
            KeyCode::Char('j') => self.tab = self.tab.next(),
            KeyCode::Char('k') => self.tab = self.tab.prev(),
            KeyCode::Enter => return Ok(Mode::Tab),
            _ => {}
        }
        Ok(Mode::TabChoose)
    }
}
