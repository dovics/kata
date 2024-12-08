use std::{sync::{Arc, Mutex}, time::Duration};

use crate::{app::Mode, kafka::KafkaGroup, theme::THEME};
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget,
    },
};
use rdkafka::consumer::{BaseConsumer, Consumer};

pub struct GroupTab {
    pub group_list: GroupList,
}

pub struct GroupList {
    pub items: Vec<KafkaGroup>,
    pub state: ListState,
}

impl GroupList {
    fn new() -> Self {
        let items = Vec::new();
        let state = ListState::default();
        Self { items, state }
    }
}

impl GroupTab {
    pub fn new() -> Self {
        let group_list = GroupList::new();

        Self { group_list }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let [group_list, group_detail] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(3)]).areas(area);
        Block::new().style(THEME.root).render(area, buf);

        self.render_left_bar(group_list, buf);
        self.render_main_area(group_detail, buf);
    }

    fn render_left_bar(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Groups").centered())
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1))
            .border_set(symbols::border::ROUNDED)
            .border_style(THEME.borders);

        let items: Vec<ListItem> = self
            .group_list
            .items
            .iter()
            .enumerate()
            .map(|(_, group)| ListItem::new(Text::from(format!("{}", group.name))))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(THEME.tabs_selected)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.group_list.state);
    }

    fn render_main_area(&mut self, area: Rect, buf: &mut Buffer) {
        let group = match self.group_list.state.selected() {
            Some(index) => &self.group_list.items[index],
            None => return,
        };

        let [group_detail, member_list] =
            Layout::vertical([Constraint::Length(3), Constraint::Fill(3)]).areas(area);
        let block = Block::new()
            .title(Line::raw(format!("Group: {}", group.name)).centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(THEME.borders)
            .padding(Padding::horizontal(1));

        let items = vec![
            Line::from(Span::raw(format!("State: {}", group.state))).style(THEME.content),
            Line::from(Span::raw(format!("Protocol: {}", group.protocol))).style(THEME.content),
            Line::from(Span::raw(format!("Protocol Type: {}", group.protocol_type)))
                .style(THEME.content),
        ];

        let paragraph = Paragraph::new(items).block(block);

        Widget::render(paragraph, group_detail, buf);

        self.render_member_list(member_list, buf, group);
    }

    fn render_member_list(&self, area: Rect, buf: &mut Buffer, group: &KafkaGroup) {
        let block = Block::new()
            .title(Line::raw("Members"))
            .borders(Borders::ALL);

        let items: Vec<ListItem> = group
            .members
            .iter()
            .map(|m| {
                ListItem::new(Text::from(format!(
                    "{} {} {}",
                    m.id, m.client_id, m.client_host
                )))
            })
            .collect();

        let list = List::new(items).block(block);
        Widget::render(list, area, buf);
    }
}

impl GroupTab {
    pub async fn refresh_matadata(&mut self, consumer: Arc<Mutex<BaseConsumer>>) -> Result<()> {
        const TIMEOUT: Duration = Duration::from_secs(5);
        let consumer = consumer.lock().unwrap();
        let group_list = consumer.fetch_group_list(None, TIMEOUT)?;
        let groups = group_list.groups();
        self.group_list.items.clear();
        for group in groups {
            let kafka_group = KafkaGroup::from(group);
            self.group_list.items.push(kafka_group);
        }
        Ok(())
    }
}

impl GroupTab {
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
        self.group_list.state.select(None);
    }

    fn select_next(&mut self) {
        self.group_list.state.select_next();
    }

    fn select_previous(&mut self) {
        self.group_list.state.select_previous();
    }

    fn select_first(&mut self) {
        self.group_list.state.select_first();
    }

    fn select_last(&mut self) {
        self.group_list.state.select_last();
    }
}
