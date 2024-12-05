use std::time::Duration;

use crate::{app::Mode, kafka::KafkaBroker, theme::THEME};
use color_eyre::{eyre::Context, Result};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    symbols,
    text::{Line, Text},
    widgets::ListState,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, Padding, StatefulWidget, Widget},
};
use rdkafka::consumer::{BaseConsumer, Consumer};
pub struct BrokerTab {
    pub broker_list: BrokerList,
}

pub struct BrokerList {
    pub items: Vec<KafkaBroker>,
    pub state: ListState,
}

impl BrokerList {
    fn new() -> Self {
        let items = Vec::new();
        let state = ListState::default();
        Self { items, state }
    }
}

impl BrokerTab {
    pub fn new() -> Self {
        let broker_list = BrokerList::new();

        Self { broker_list }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        Block::new().style(THEME.root).render(area, buf);
        self.render_left_bar(area, buf);
    }

    fn render_left_bar(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Brokers").centered())
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1))
            .border_set(symbols::border::ROUNDED)
            .border_style(THEME.borders);

        let items: Vec<ListItem> = self
            .broker_list
            .items
            .iter()
            .enumerate()
            .map(|(_, broker)| {
                ListItem::new(Text::from(format!("{}:{}", broker.host, broker.port)))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(THEME.tabs_selected)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.broker_list.state);
    }
}

impl BrokerTab {
    pub fn refresh_matadata(&mut self, consumer: &BaseConsumer) -> Result<()> {
        const TIMEOUT: Duration = Duration::from_secs(5);
        let metadata = consumer
            .fetch_metadata(None, TIMEOUT)
            .wrap_err("Failed to fetch metadata")?;
        self.broker_list.items.clear();

        for broker in metadata.brokers() {
            let kafka_broker = KafkaBroker::from(broker);
            self.broker_list.items.push(kafka_broker);
        }
        Ok(())
    }
}

impl BrokerTab {
    pub fn handle_key_press(&mut self, key: &KeyEvent) -> Result<Mode> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => return Ok(Mode::TabChoose),
            KeyCode::Char('r') => return Ok(Mode::Refresh),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),

            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            _ => {}
        };

        Ok(Mode::Tab)
    }

    fn select_none(&mut self) {
        self.broker_list.state.select(None);
    }

    fn select_next(&mut self) {
        self.broker_list.state.select_next();
    }

    fn select_previous(&mut self) {
        self.broker_list.state.select_previous();
    }

    fn select_first(&mut self) {
        self.broker_list.state.select_first();
    }

    fn select_last(&mut self) {
        self.broker_list.state.select_last();
    }
}
