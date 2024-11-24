use color_eyre::Result;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::Color,
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, Padding, Paragraph, StatefulWidget,
        Widget,
    },
};

use crate::{
    app::{App, Mode, TopicTab},
    kafka::KafkaTopic,
    theme::THEME,
};
impl App {
    pub fn render_normal_page(&mut self, area: Rect, buf: &mut Buffer) {
        let [title_bar, main_area, bottom_bar] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [topic_list, topic_detail] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Fill(3)]).areas(main_area);
        Block::new().style(THEME.root).render(area, buf);
        App::render_title_bar(title_bar, buf);
        App::render_bottom_bar(bottom_bar, buf);
        self.render_left_bar(topic_list, buf);

        self.render_selected_item(topic_detail, buf);
    }

    fn render_title_bar(area: Rect, buf: &mut Buffer) {
        Paragraph::new("Kafka TUI")
            .style(THEME.app_title)
            .centered()
            .render(area, buf);
    }

    fn render_bottom_bar(area: Rect, buf: &mut Buffer) {
        let keys = [
            ("K/↑", "Up"),
            ("J/↓", "Down"),
            ("Q/Esc", "Quit"),
            ("g/G", "First/Last"),
        ];
        let spans: Vec<Span> = keys
            .iter()
            .flat_map(|(key, desc)| {
                let key = Span::styled(format!(" {key} "), THEME.key_binding.key);
                let desc = Span::styled(format!(" {desc} "), THEME.key_binding.description);
                [key, desc]
            })
            .collect();
        Line::from(spans)
            .centered()
            .style((Color::Indexed(236), Color::Indexed(232)))
            .render(area, buf);
    }

    fn render_left_bar(&mut self, area: Rect, buf: &mut Buffer) {
        let [broker_list, topic_list] = Layout::vertical([
            Constraint::Length(self.brokers.len() as u16 + 2),
            Constraint::Fill(1),
        ])
        .areas(area);

        self.render_broker_list(broker_list, buf);
        self.render_topic_list(topic_list, buf);
    }

    fn render_broker_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Brokers").centered())
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1))
            .border_set(symbols::border::ROUNDED)
            .border_style(THEME.borders);

        let items: Vec<ListItem> = self
            .brokers
            .iter()
            .map(|broker| ListItem::new(Text::from(format!("{}:{}", broker.host, broker.port))))
            .collect();

        let list = List::new(items).block(block);

        Widget::render(list, area, buf);
    }

    fn render_topic_list(&mut self, area: Rect, buf: &mut Buffer) {
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

        match self.topic_tab {
            TopicTab::Info => self.render_topic_info(area, buf, topic),
            TopicTab::Messages => self.render_topic_messages(area, buf, topic),
            TopicTab::Send => self.render_topic_send(area, buf, topic),
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

impl App {
    pub fn handle_key_press_normal(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('r') => self.refresh_metadata()?,
            KeyCode::Char('s') => self.mode = Mode::Input,

            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('l') | KeyCode::Right => self.topic_detail(),
            _ => {}
        };
        
        Ok(())
    }

    fn select_none(&mut self) {
        match self.mode {
            Mode::Normal => self.topic_list.state.select(None),
            Mode::TopicInfo => self.mode = Mode::Normal,
            _ => {}
        }
    }

    fn select_next(&mut self) {
        match self.mode {
            Mode::Normal => self.topic_list.state.select_next(),
            Mode::TopicInfo => match self.topic_tab {
                TopicTab::Info => self.topic_tab = TopicTab::Messages,
                TopicTab::Messages => self.topic_tab = TopicTab::Send,
                TopicTab::Send => self.topic_tab = TopicTab::Info,
            },
            _ => {}
        }
    }

    fn select_previous(&mut self) {
        match self.mode {
            Mode::Normal => self.topic_list.state.select_previous(),
            Mode::TopicInfo => match self.topic_tab {
                TopicTab::Info => self.topic_tab = TopicTab::Send,
                TopicTab::Messages => self.topic_tab = TopicTab::Info,
                TopicTab::Send => self.topic_tab = TopicTab::Messages,
            },
            _ => {}
        }
    }

    fn topic_detail(&mut self) {
        if self.topic_list.state.selected().is_some() {
            self.mode = Mode::TopicInfo;
        }
    }

    fn select_first(&mut self) {
        self.topic_list.state.select_first();
    }

    fn select_last(&mut self) {
        self.topic_list.state.select_last();
    }
}
