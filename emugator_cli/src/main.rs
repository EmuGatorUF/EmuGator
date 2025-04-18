// The command-line interface was implemented referencing Ratatui's gauge example: https://ratatui.rs/examples/widgets/gauge/

use clap::Parser;
use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::palette::tailwind;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, Padding, Paragraph, Widget};
use std::time::Duration;
use tester::TestInfo;

mod tester;

const GAUGE_COLOR: Color = tailwind::BLUE.c800;
const CUSTOM_LABEL_COLOR: Color = tailwind::SLATE.c200;
const GAUGE_LABEL_COLOR: Color = tailwind::ORANGE.c800;

#[derive(Debug, Default)]
struct App {
    state: AppState,
    tester: TestInfo,
    ending_msg: Option<String>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum AppState {
    #[default]
    Started,
    Quitting,
}

fn main() {
    let args = tester::Arguments::parse();
    match args.command {
        tester::Command::New(new_args) => {
            tester::new_project(new_args);
        }
        tester::Command::Test(test_args) => {
            tests_with_ratatui(test_args);
        }
    }
}

fn tests_with_ratatui(test_args: tester::TestArgs) {
    let _ = color_eyre::install();
    let terminal = ratatui::init();
    let _ = App::default().run(terminal, test_args);
    ratatui::restore();
}

impl App {
    fn run(mut self, mut terminal: DefaultTerminal, test_args: tester::TestArgs) -> Result<()> {
        self.tester = tester::TestInfo::default();
        self.tester.prepare_to_test(test_args);

        while self.state != AppState::Quitting {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
            self.update();
        }
        Ok(())
    }

    fn update(&mut self) {
        if self.state != AppState::Started {
            return;
        }

        self.run_tests();
    }

    fn handle_events(&mut self) -> Result<()> {
        let timeout = Duration::from_secs_f32(1.0 / 20.0);
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char(' ') | KeyCode::Enter => self.quit(),
                        KeyCode::Char('q') | KeyCode::Esc => self.quit(),
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn quit(&mut self) {
        self.state = AppState::Quitting;
    }

    fn run_tests(&mut self) {
        if !self.tester.run_curr_test() {
            self.ending_msg = Some(self.tester.finish_up());
        }
    }

    fn render_gauges(&self, area: Rect, buf: &mut Buffer) {
        let prog_title = format!("Current Program: {}", self.tester.current_program());
        let mut prog_area = area;
        prog_area.height = area.height / 2;
        make_gague(
            title_block(&prog_title),
            self.tester.curr_prog,
            self.tester.num_programs(),
        )
        .render(prog_area, buf);

        let test_title = format!("Current Test: {}", self.tester.current_test());
        let mut test_area = area;
        test_area.y = area.y + area.height / 2;
        test_area.height = area.height / 2;
        make_gague(
            title_block(&test_title),
            self.tester.curr_test,
            self.tester.num_tests(),
        )
        .render(test_area, buf);
    }

    fn render_ending_text(&self, area: Rect, buf: &mut Buffer) {
        if let Some(ending_msg) = &self.ending_msg {
            Paragraph::new(ending_msg.clone())
                .alignment(Alignment::Center)
                .fg(GAUGE_LABEL_COLOR)
                .render(area, buf);
        }
    }
}

impl Widget for &App {
    #[allow(clippy::similar_names)]
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min};
        let layout = Layout::vertical([Length(2), Min(0), Length(2), Length(1)]);
        let [header_area, gauge_area, middle_area, footer_area] = layout.areas(area);

        render_header(header_area, buf);
        render_footer(footer_area, buf);
        if self.ending_msg.is_some() {
            self.render_ending_text(middle_area, buf);
        } else {
            self.render_gauges(gauge_area, buf);
        }
    }
}

fn render_header(area: Rect, buf: &mut Buffer) {
    Paragraph::new("EmuGator Command Line Auto Grader")
        .bold()
        .alignment(Alignment::Center)
        .fg(CUSTOM_LABEL_COLOR)
        .render(area, buf);
}

fn render_footer(area: Rect, buf: &mut Buffer) {
    Paragraph::new("Press ENTER or Q to quit.")
        .alignment(Alignment::Center)
        .fg(CUSTOM_LABEL_COLOR)
        .bold()
        .render(area, buf);
}

fn title_block(title: &str) -> Block {
    let title = Line::from(title).centered();
    Block::new()
        .borders(Borders::NONE)
        .padding(Padding::vertical(1))
        .title(title)
        .fg(CUSTOM_LABEL_COLOR)
}

fn make_gague<'a>(title: Block<'a>, curr: usize, total: usize) -> Gauge<'a> {
    let ratio = if total == 0 {
        0.0
    } else {
        curr as f64 / total as f64
    };
    let label = Span::styled(
        format!("{}/{}", curr, total),
        Style::new().italic().bold().fg(GAUGE_LABEL_COLOR),
    );
    Gauge::default()
        .block(title)
        .gauge_style(GAUGE_COLOR)
        .ratio(ratio)
        .label(label)
}
