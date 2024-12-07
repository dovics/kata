use std::time::{Duration, SystemTime};

use crate::{app::Mode, kafka::KafkaTopic, tabs::topic_send::TopicSendForm, theme::THEME};
use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, StatefulWidget,
        Widget,
    },
};
use rdkafka::{
    admin::{AdminClient, AdminOptions, NewTopic},
    client::DefaultClientContext,
    consumer::{BaseConsumer, Consumer},
    producer::FutureProducer,
};
pub struct TopicTab {
    pub topic_list: TopicList,
    pub topic_page: TopicPage,

    send_form: TopicSendForm,

    err: Option<String>,
    err_time: Option<SystemTime>,
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
    SendEdit,
}

impl TopicTab {
    pub fn new() -> Self {
        let topic_list = TopicList::new();
        let topic_page = TopicPage::default();
        let send_form = TopicSendForm::new("");
        Self {
            topic_list,
            topic_page,
            send_form,

            err: None,
            err_time: None,
        }
    }

    fn set_error(&mut self, error: String) {
        self.err = Some(error.clone());
        self.err_time = Some(SystemTime::now());
    }

    fn clear_error(&mut self) {
        self.err = None;
        self.err_time = None;
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

    fn render_selected_item(&mut self, area: Rect, buf: &mut Buffer) {
        let topic = match self.topic_list.state.selected() {
            Some(index) => &self.topic_list.items[index],
            None => return,
        };

        match self.topic_page {
            TopicPage::Normal | TopicPage::Info => self.render_topic_info(area, buf, topic),
            TopicPage::Messages => self.render_topic_messages(area, buf, topic),
            TopicPage::Send | TopicPage::SendEdit => self.render_topic_send(area, buf),
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

    fn render_topic_send(&mut self, area: Rect, buf: &mut Buffer) {
        let current_topic = &self.topic_list.items[self.topic_list.state.selected().unwrap()].name;
        if self.send_form.get_topic() != *current_topic {
            self.send_form.set_topic(current_topic);
            self.send_form.empty();
        }

        self.send_form.render(area, buf);
    }

    pub fn bottom_bar_spans(&mut self) -> Vec<Span> {
        if let Some(err_time) = &self.err_time {
            if SystemTime::now()
                .duration_since(*err_time)
                .unwrap()
                .as_secs()
                > 5
            {
                self.clear_error();
            } else if let Some(err) = &self.err {
                return vec![Span::raw(err).style(THEME.error)];
            }
        }

        let keys = [
            ("K/↑", "Up"),
            ("J/↓", "Down"),
            ("Q/Esc", "Quit"),
            ("g/G", "First/Last"),
        ];

        keys.iter()
            .flat_map(|(key, desc)| {
                let key = Span::styled(format!(" {key} "), THEME.key_binding.key);
                let desc = Span::styled(format!(" {desc} "), THEME.key_binding.description);
                [key, desc]
            })
            .collect()
    }
}

impl TopicTab {
    pub fn refresh_matadata(&mut self, consumer: &BaseConsumer) {
        const TIMEOUT: Duration = Duration::from_secs(5);
        match consumer.fetch_metadata(None, TIMEOUT) {
            Ok(metadata) => {
                self.topic_list.items.clear();
                for topic in metadata.topics() {
                    let mut kafka_topic = KafkaTopic::from(topic);
                    for partition in &mut kafka_topic.partitions {
                        match consumer.fetch_watermarks(&kafka_topic.name, partition.id, TIMEOUT) {
                            Ok((low, high)) => {
                                partition.low = low;
                                partition.high = high;
                            }
                            Err(e) => {
                                self.set_error(e.to_string());
                                return;
                            }
                        }
                    }
                    self.topic_list.items.push(kafka_topic);
                }
            }
            Err(e) => {
                self.set_error(e.to_string());
                return;
            }
        };
    }

    pub async fn create_topic(&mut self, admin: &AdminClient<DefaultClientContext>) {
        let topic = NewTopic::new("test", 1, rdkafka::admin::TopicReplication::Fixed(1));
        if let Err(e) = admin.create_topics(&[topic], &AdminOptions::new()).await {
            self.set_error(e.to_string());
        }
    }
}

impl TopicTab {
    pub async fn handle_key_press(
        &mut self,
        key: &KeyEvent,
        producer: &FutureProducer,
        admin: &AdminClient<DefaultClientContext>,
    ) -> Result<Mode> {
        if self.topic_page == TopicPage::SendEdit {
            self.topic_page = match self.send_form.handle_key_press(key, producer).await {
                Ok(page) => page,
                Err(e) => {
                    self.set_error(e.to_string());
                    TopicPage::SendEdit
                }
            };
            return Ok(Mode::Tab);
        }

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => match self.topic_page {
                TopicPage::Normal => return Ok(Mode::TabChoose),
                _ => self.topic_page = TopicPage::Normal,
            },
            KeyCode::Char('r') => return Ok(Mode::Refresh),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),

            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('l') | KeyCode::Right => self.topic_detail(),
            KeyCode::Char('n') => {
                self.create_topic(admin).await;
            }
            // KeyCode::Char('d') => self.delete_topic(producer),
            KeyCode::Enter => {
                if self.topic_page == TopicPage::Send {
                    self.topic_page = TopicPage::SendEdit;
                } else {
                    self.topic_detail();
                }
            }
            _ => {}
        };

        Ok(Mode::Tab)
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
            TopicPage::Send | TopicPage::SendEdit => self.topic_page = TopicPage::Info,
        }
    }

    fn select_previous(&mut self) {
        match self.topic_page {
            TopicPage::Normal => self.topic_list.state.select_previous(),

            TopicPage::Info => self.topic_page = TopicPage::Send,
            TopicPage::Messages => self.topic_page = TopicPage::Info,
            TopicPage::Send | TopicPage::SendEdit => self.topic_page = TopicPage::Messages,
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
