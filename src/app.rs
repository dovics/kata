use crate::{tabs::{BrokerTab, TopicTab}, theme::THEME};
use color_eyre::{eyre::Context, Result};
use crossterm::event::{self};
use ratatui::{
    buffer::Buffer, crossterm::event::{Event, KeyEventKind}, layout::{Constraint, Layout, Rect}, style::Color, text::{Line, Span}, widgets::{Paragraph, Widget}, DefaultTerminal, Frame
};
use rdkafka::{config::ClientConfig, consumer::BaseConsumer, producer::BaseProducer};
use std::time::Duration;

const TIMEOUT: Duration = Duration::from_secs(10);

pub struct App {
    pub mode: Mode,

    consumer: BaseConsumer,
    producer: BaseProducer,

    topic_tab: TopicTab,
    broker_tab: BrokerTab,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Topic,
    Group,
    Broker,
    Quit,
    Refresh,
}

impl App {
    pub fn new(brokers: Vec<String>) -> Result<Self> {
        let mut config = ClientConfig::new();
        let config = config.set("bootstrap.servers", &brokers.join(","));
        let consumer = config.create().wrap_err("Consumer creation failed")?;
        let producer = config.create().wrap_err("Producer creation failed")?;

        let topic_tab = TopicTab::new();
        let broker_tab = BrokerTab::new();
        Ok(Self {
            mode: Mode::default(),
            consumer,
            producer,
            topic_tab,
            broker_tab,
        })
    }

    pub fn refresh_matadata(&mut self) -> Result<()> {
        match self.mode {
            Mode::Topic => self.topic_tab.refresh_matadata(&self.consumer),
            Mode::Broker => self.broker_tab.refresh_matadata(&self.consumer),
            _ => Ok(()),
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.refresh_matadata()?;
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

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        let buf = frame.buffer_mut();
        let [title_bar, main_area, bottom_bar] = Layout::vertical([
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);
        Self::render_title_bar(title_bar, buf);
        Self::render_bottom_bar(bottom_bar, buf);

        match self.mode {
            Mode::Topic => self.topic_tab.render(main_area, buf),
            Mode::Broker => self.broker_tab.render(main_area, buf),
            _ => {}
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        let timeout = Duration::from_secs_f64(1.0 / 50.0);
        if !event::poll(timeout)? {
            return Ok(());
        }
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                self.mode = match self.mode {
                    Mode::Topic => self.topic_tab.handle_key_press(key)?,
                    Mode::Broker => self.broker_tab.handle_key_press(key)?,
                    _ => self.mode,
                };
            }
            _ => {}
        }
        Ok(())
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
}
