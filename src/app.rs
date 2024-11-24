use crate::kafka::{KafkaBroker, KafkaTopic};
use crate::theme::THEME;
use color_eyre::{eyre::Context, Result};
use crossterm::event;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::Color,
    symbols,
    text::{Line, Span, Text},
    widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Padding, Paragraph,
        StatefulWidget, Widget,
    },
    DefaultTerminal, Frame,
};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, Consumer};
use rdkafka::metadata::Metadata;
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(10);

pub struct App {
    mode: Mode,

    metadata: Metadata,

    brokers: Vec<KafkaBroker>,
    topic_list: TopicList,
}

struct TopicList {
    items: Vec<KafkaTopic>,
    state: ListState,
}

impl TopicList {
    fn new() -> Self {
        let items = Vec::new();
        let state = ListState::default();
        Self { items, state }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum Mode {
    #[default]
    Running,
    // Destroy,
    Quit,
}

impl App {
    pub fn new(brokers: Vec<String>) -> Self {
        let consumer: BaseConsumer = ClientConfig::new()
            .set("bootstrap.servers", &brokers.join(","))
            .create()
            .expect("Consumer creation failed");

        let metadata = consumer
            .fetch_metadata(None, TIMEOUT)
            .expect("Failed to fetch metadata");

        Self {
            mode: Mode::default(),
            metadata,
            brokers: Vec::new(),
            topic_list: TopicList::new(),
        }
    }

    pub fn refresh_metadata(&mut self) -> Result<()> {
        self.brokers.clear();
        for broker in self.metadata.brokers() {
            self.brokers.push(KafkaBroker::from(broker));
        }

        self.topic_list.items.clear();
        for topic in self.metadata.topics() {
            self.topic_list.items.push(KafkaTopic::from(topic));
        }

        Ok(())
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.refresh_metadata()?;
        while self.is_running() {
            terminal
                .draw(|frame| self.draw(frame))
                .wrap_err("terminal.draw")?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn is_running(&self) -> bool {
        self.mode != Mode::Quit
    }

    /// Draw a single frame of the app.
    fn draw(&mut self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
        // if self.mode == Mode::Destroy {
        //     destroy::destroy(frame);
        // }
    }

    fn handle_events(&mut self) -> Result<()> {
        let timeout = Duration::from_secs_f64(1.0 / 50.0);
        if !event::poll(timeout)? {
            return Ok(());
        }
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key_press(key),
            _ => {}
        }
        Ok(())
    }

    fn handle_key_press(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.mode = Mode::Quit,
            KeyCode::Char('h') | KeyCode::Left => self.select_none(),
            KeyCode::Char('j') | KeyCode::Down => self.select_next(),
            KeyCode::Char('k') | KeyCode::Up => self.select_previous(),
            KeyCode::Char('g') | KeyCode::Home => self.select_first(),
            KeyCode::Char('G') | KeyCode::End => self.select_last(),
            KeyCode::Char('r') => self.refresh_metadata().unwrap(),
            _ => {}
        };
    }

    fn select_none(&mut self) {
        self.topic_list.state.select(None);
    }

    fn select_next(&mut self) {
        self.topic_list.state.select_next();
    }
    fn select_previous(&mut self) {
        self.topic_list.state.select_previous();
    }

    fn select_first(&mut self) {
        self.topic_list.state.select_first();
    }

    fn select_last(&mut self) {
        self.topic_list.state.select_last();
    }
}

impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
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
        self.render_topic_list(topic_list, buf);

        self.render_selected_item(topic_detail, buf);
    }
}

impl App {
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

    fn render_topic_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw("Topic List").centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
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

        let block = Block::new()
            .title(Line::raw(format!("Topic: {}", topic.name)).centered())
            .borders(Borders::TOP)
            .border_set(symbols::border::EMPTY)
            .border_style(THEME.borders)
            .padding(Padding::horizontal(1));

        let items: Vec<ListItem> = topic
            .partitions
            .iter()
            .enumerate()
            .map(|(_, p)| {
                let content = Text::from(vec![
                    Line::from(Span::raw(format!("Partition: {}", p.id))).style(THEME.content),
                    Line::from(Span::raw(format!("Leader: {}", p.leader))).style(THEME.content),
                    Line::from(Span::raw(format!("Replicas: {:?}", p.replicas)))
                        .style(THEME.content),
                    Line::from(Span::raw(format!("ISR: {:?}", p.isr))).style(THEME.content),
                ]);
                ListItem::new(content).style(THEME.borders)
            })
            .collect();

        let list = List::new(items).block(block);

        Widget::render(list, area, buf);
    }
}
