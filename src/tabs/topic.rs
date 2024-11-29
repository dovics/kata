use std::time::Duration;

use crate::{app::Mode, kafka::KafkaTopic, theme::THEME};
use color_eyre::{
    eyre::{eyre, Context},
    Result,
};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    symbols,
    text::{Line, Span, Text},
    widgets::ListState,
    widgets::{Block, Borders, HighlightSpacing, List, ListItem, Padding, StatefulWidget, Widget},
};
use rdkafka::consumer::{BaseConsumer, Consumer};
pub struct TopicTab {
    pub topic_list: TopicList,
    pub topic_page: TopicPage,
}

pub struct TopicList {
    pub items: Vec<KafkaTopic>,
    pub state: ListState,
}

impl TopicList {
    fn new() -> Self {
        let items = Vec::new();
        let state = ListState::default();
        Self { items, state }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum TopicPage {
    #[default]
    Normal,
    Info,
    Messages,
    Send,
}

impl TopicTab {
    pub fn new() -> Self {
        let topic_list = TopicList::new();
        let topic_page = TopicPage::default();
        Self {
            topic_list,
            topic_page,
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        let [topic_list, topic_detail] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(3)]).areas(area);
        Block::new().style(THEME.root).render(area, buf);
        self.render_left_bar(topic_list, buf);

        self.render_selected_item(topic_detail, buf);
    }

    fn render_left_bar(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Topics").centered())
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1))
            .border_set(symbols::border::ROUNDED)
            .border_style(THEME.borders);

        let items: Vec<ListItem> = self
            .topic_list
            .items
            .iter()
            .enumerate()
            .map(|(_, topic)| ListItem::new(Text::from(topic.name.clone())))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(THEME.tabs_selected)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.topic_list.state);
    }

    fn render_selected_item(&self, area: Rect, buf: &mut Buffer) {
        let topic = match self.topic_list.state.selected() {
            Some(index) => &self.topic_list.items[index],
            None => return,
        };

        match self.topic_page {
            TopicPage::Normal | TopicPage::Info => self.render_topic_info(area, buf, topic),
            TopicPage::Messages => self.render_topic_messages(area, buf, topic),
            TopicPage::Send => self.render_topic_send(area, buf, topic),
        }
    }

    fn render_topic_info(&self, area: Rect, buf: &mut Buffer, topic: &KafkaTopic) {
        let block = Block::new()
            .title(Line::raw(format!("Topic: {}", topic.name)).centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(THEME.borders)
            .padding(Padding::horizontal(1));

        let items: Vec<ListItem> = topic
            .partitions
            .iter()
            .enumerate()
            .map(|(_, p)| {
                let content = Text::from(vec![
                    Line::from(Span::raw(format!(
                        "Partition: {}    Leader: {}",
                        p.id, p.leader
                    )))
                    .style(THEME.content),
                    Line::from(Span::raw(format!(
                        "  Replicas: {:?}  ISR: {:?}",
                        p.replicas, p.isr
                    )))
                    .style(THEME.content),
                    Line::from(Span::raw(format!("  Low: {}    High: {}", p.low, p.high)))
                        .style(THEME.content),
                ]);
                ListItem::new(content).style(THEME.borders)
            })
            .collect();

        let list = List::new(items).block(block);

        Widget::render(list, area, buf);
    }

    fn render_topic_messages(&self, area: Rect, buf: &mut Buffer, topic: &KafkaTopic) {
        let block = Block::new()
            .title(Line::raw(format!("Messages for {}", topic.name)).centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(THEME.borders)
            .padding(Padding::horizontal(1));

        Widget::render(block, area, buf);
    }

    fn render_topic_send(&self, area: Rect, buf: &mut Buffer, topic: &KafkaTopic) {
        let block = Block::new()
            .title(Line::raw(format!("Send Message to {}", topic.name)).centered())
            .borders(Borders::ALL)
            .border_set(symbols::border::ROUNDED)
            .border_style(THEME.borders)
            .padding(Padding::horizontal(1));

        Widget::render(block, area, buf);
    }
}

impl TopicTab {
    pub fn refresh_matadata(&mut self, consumer: &BaseConsumer) -> Result<()> {
        const TIMEOUT: Duration = Duration::from_secs(5);
        let metadata = consumer
            .fetch_metadata(None, TIMEOUT)
            .wrap_err("Failed to fetch metadata")?;
        self.topic_list.items.clear();
        for topic in metadata.topics() {
            let mut kafka_topic = KafkaTopic::from(topic);

            for partition in &mut kafka_topic.partitions {
                let (low, high) = consumer
                    .fetch_watermarks(&kafka_topic.name, partition.id, TIMEOUT)
                    .map_err(|e| eyre!(e))
                    .wrap_err("Failed to fetch watermarks")?;
                partition.low = low;
                partition.high = high;
            }
            self.topic_list.items.push(kafka_topic);
        }

        Ok(())
    }
}

impl TopicTab {
    pub fn handle_key_press(&mut self, key: KeyEvent) -> Result<Mode> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => match self.topic_page {
                TopicPage::Normal => return Ok(Mode::Quit),
                _ => self.topic_page = TopicPage::Normal,
            },
            KeyCode::Char('r') => return Ok(Mode::Refresh),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),

            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('l') | KeyCode::Right => self.topic_detail(),
            _ => {}
        };

        Ok(Mode::Topic)
    }

    fn select_none(&mut self) {
        match self.topic_page {
            TopicPage::Normal => self.topic_list.state.select(None),
            _ => self.topic_page = TopicPage::Normal,
        }
    }

    fn select_next(&mut self) {
        match self.topic_page {
            TopicPage::Normal => self.topic_list.state.select_next(),

            TopicPage::Info => self.topic_page = TopicPage::Messages,
            TopicPage::Messages => self.topic_page = TopicPage::Send,
            TopicPage::Send => self.topic_page = TopicPage::Info,
        }
    }

    fn select_previous(&mut self) {
        match self.topic_page {
            TopicPage::Normal => self.topic_list.state.select_previous(),

            TopicPage::Info => self.topic_page = TopicPage::Send,
            TopicPage::Messages => self.topic_page = TopicPage::Info,
            TopicPage::Send => self.topic_page = TopicPage::Messages,
        }
    }

    fn topic_detail(&mut self) {
        if self.topic_list.state.selected().is_some() {
            self.topic_page = TopicPage::Info;
        }
    }

    fn select_first(&mut self) {
        self.topic_list.state.select_first();
    }

    fn select_last(&mut self) {
        self.topic_list.state.select_last();
    }
}
