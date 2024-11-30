use crate::{
    tabs::{BrokerTab, GroupTab, Tab, TopicTab},
    theme::THEME,
};
use color_eyre::{eyre::Context, Result};
use crossterm::event::{self};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::Color,
    text::{Line, Span},
    widgets::{Paragraph, Tabs, Widget},
    DefaultTerminal, Frame,
};
use rdkafka::{config::ClientConfig, consumer::BaseConsumer, producer::BaseProducer};
use std::time::Duration;
use strum::IntoEnumIterator;

pub struct App {
    mode: Mode,
    pub tab: Tab,
    consumer: BaseConsumer,
    producer: BaseProducer,

    broker_tab: BrokerTab,
    group_tab: GroupTab,
    topic_tab: TopicTab,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    #[default]
    TabChoose,
    Tab,
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
        let group_tab = GroupTab::new();
        Ok(Self {
            mode: Mode::default(),
            tab: Tab::default(),
            consumer,
            producer,
            topic_tab,
            broker_tab,
            group_tab,
        })
    }

    pub fn refresh_matadata(&mut self) -> Result<()> {
        let tabs = if self.mode == Mode::Tab {
            vec![self.tab]
        } else {
            Tab::iter().collect()
        };

        for tab in tabs {
            match tab {
                Tab::Topic => self.topic_tab.refresh_matadata(&self.consumer)?,
                Tab::Group => self.group_tab.refresh_matadata(&self.consumer)?,
                Tab::Broker => self.broker_tab.refresh_matadata(&self.consumer)?,
            }
        }
        Ok(())
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
        let buf: &mut Buffer = frame.buffer_mut();
        let [title_bar, main_area, bottom_bar] = Layout::vertical([
            Constraint::Length(2),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        self.render_title_bar(title_bar, buf);
        self.render_bottom_bar(bottom_bar, buf);

        match self.tab {
            Tab::Topic => self.topic_tab.render(main_area, buf),
            Tab::Group => self.group_tab.render(main_area, buf),
            Tab::Broker => self.broker_tab.render(main_area, buf),
        }
    }

    fn handle_events(&mut self) -> Result<()> {
        let timeout = Duration::from_secs_f64(1.0 / 50.0);
        if !event::poll(timeout)? {
            return Ok(());
        }

        self.mode = match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match self.mode {
                Mode::TabChoose => self.handle_tab_select(key)?,
                Mode::Tab => match self.tab {
                    Tab::Topic => self.topic_tab.handle_key_press(key, &self.producer)?,
                    Tab::Group => self.group_tab.handle_key_press(key)?,
                    Tab::Broker => self.broker_tab.handle_key_press(key)?,
                },
                _ => self.mode,
            },
            _ => self.mode,
        };

        Ok(())
    }

    fn render_title_bar(&mut self, area: Rect, buf: &mut Buffer) {
        let [title, tabs] =
            Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(area);
        Paragraph::new("Kafka TUI")
            .style(THEME.app_title)
            .centered()
            .render(title, buf);

        let tab_titles = Tab::iter().map(Tab::title);
        Tabs::new(tab_titles)
            .style(THEME.tabs)
            .highlight_style(if self.mode == Mode::TabChoose {
                THEME.tabs_selected
            } else {
                THEME.tabs
            })
            .select(self.tab as usize)
            .divider(" ")
            .padding("", "")
            .render(tabs, buf);
    }

    fn render_bottom_bar(&mut self, area: Rect, buf: &mut Buffer) {
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
