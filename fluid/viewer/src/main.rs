use std::collections::HashMap;
use std::io;
use std::rc::Rc;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{self, disable_raw_mode, enable_raw_mode};
use crossterm::ExecutableCommand;
use dynec::{tracer, Entity, World};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::widgets::{Block, Chart, Dataset};
use ratatui::{Frame, Terminal};
use traffloat_fluid::container::{self, Container};
use traffloat_fluid::{Pressure, TypeDef, TypeDefs, Viscosity, Volume};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

fn main() -> Result<()> {
    let mut world = dynec::new([
        Box::new(traffloat_fluid::Bundle) as Box<dyn dynec::Bundle>,
        Box::new(InitBundle),
    ]);

    enable_raw_mode()?;
    io::stdout().execute(terminal::EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    struct DeferReset;
    impl Drop for DeferReset {
        fn drop(&mut self) {
            _ = disable_raw_mode();
            _ = io::stdout().execute(terminal::LeaveAlternateScreen);
        }
    }

    let _defer = DeferReset;

    tui_logger::init_logger(log::LevelFilter::Trace)?;

    let mut ui_state = UiState::default();

    let mut container_stats = HashMap::new();
    for x in 0..=2 {
        for y in 0..=2 {
            container_stats.insert((x, y), ContainerStats::default());
        }
    }

    for time in 0.. {
        world.execute(&tracer::Log(log::Level::Info));

        terminal.draw(|frame| {
            let cols = Layout::new(Direction::Horizontal, [Constraint::Ratio(1, 5); 5])
                .split(frame.size());
            let cells: Vec<Rc<[Rect]>> = cols
                .into_iter()
                .map(|&rect| {
                    Layout::new(Direction::Vertical, [Constraint::Ratio(1, 5); 5]).split(rect)
                })
                .collect();
            for x in [0, 2, 4] {
                for y in [0, 2, 4] {
                    render_container(
                        frame,
                        cells[x][y],
                        (x / 2, y / 2),
                        &mut container_stats,
                        &mut world,
                        time.into(),
                    );
                }
                for y in [1, 3] {
                    render_pipe(frame, cells[x][y], (x / 2, y / 2), (x / 2, y / 2 + 1))
                }
            }
            for x in [1, 3] {
                for y in [0, 2, 4] {
                    render_pipe(frame, cells[x][y], (x / 2, y / 2), (x / 2 + 1, y / 2))
                }
            }
            frame.render_widget(
                TuiLoggerWidget::default()
                    .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
                    .output_file(false)
                    .output_line(false)
                    .output_target(false)
                    .output_separator(' '),
                cells[3][3],
            );
        })?;

        ui_state.handle_events()?;
        if ui_state.stopped {
            break;
        }
    }

    Ok(())
}

#[derive(Default)]
struct ContainerStats {
    pressure: Vec<(f64, f64)>,
    volume:   Vec<(f64, f64)>,
}

const BACKLOG_SIZE: usize = 40;

impl ContainerStats {
    fn update_datasets(
        &mut self,
        components: &mut dynec::world::Components,
        entity: &Entity<Container>,
        now: f64,
    ) -> Vec<Dataset<'_>> {
        let pressure = components
            .get_simple_storage::<_, container::CurrentPressure>()
            .get(entity)
            .pressure
            .quantity;
        let volume = components
            .get_simple_storage::<_, container::CurrentVolume>()
            .get(entity)
            .volume
            .quantity;

        let mut datasets = Vec::new();
        for (name, style, last, vec) in [
            ("pressure", Style::default().cyan(), pressure, &mut self.pressure),
            ("volume", Style::default().red(), volume, &mut self.volume),
        ] {
            if last.is_finite() {
                if vec.len() >= BACKLOG_SIZE * 2 {
                    vec.drain(0..(vec.len() - BACKLOG_SIZE + 1)).for_each(|_| ());
                }
                vec.push((now, last));
            } else {
                log::warn!("invalid {name} value {last}");
            }

            datasets.push(Dataset::default().name(name).style(style).data(&vec[..]));
        }

        datasets
    }
}

fn render_container(
    frame: &mut Frame<'_>,
    rect: Rect,
    (x, y): (usize, usize),
    containers: &mut HashMap<(usize, usize), ContainerStats>,
    world: &mut World,
    now: f64,
) {
    let entities = world.sync_globals.get_mut::<Entities>();

    let chart = Chart::new(containers.get_mut(&(x, y)).unwrap().update_datasets(
        &mut world.components,
        entities.containers.get(&(x, y)).unwrap(),
        now,
    ))
    .block(Block::bordered().title(format!("({x}, {y})")));
    frame.render_widget(chart, rect.inner(&Margin::new(3, 0)));
}

fn render_pipe(
    frame: &mut Frame<'_>,
    rect: Rect,
    (x1, y1): (usize, usize),
    (x2, y2): (usize, usize),
) {
    let chart = Chart::new(Vec::new())
        .block(Block::bordered().title(format!("({x1}, {y1}) -> ({x2}, {y2})")));
    frame.render_widget(chart, rect.inner(&Margin::new(6, 1)));
}

struct UiState {
    stopped: bool,
}

impl Default for UiState {
    fn default() -> Self { Self { stopped: false } }
}

impl UiState {
    fn handle_events(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
                    self.stopped = true;
                }
            }
        }

        Ok(())
    }
}

struct InitBundle;

#[dynec::global]
#[derive(Default)]
struct Entities {
    #[entity]
    containers: HashMap<(usize, usize), Entity<Container>>,
}

impl dynec::Bundle for InitBundle {
    fn register(&mut self, builder: &mut dynec::world::Builder) {
        builder.global(TypeDefs {
            defs: vec![TypeDef {
                viscosity:              Viscosity { quantity: 0.5 },
                vacuum_specific_volume: 10.,
                critical_pressure:      Pressure { quantity: 10. },
            }],
        });
        builder.global(Entities::default());
    }

    fn populate(&mut self, world: &mut dynec::World) {
        for x in 0..=2 {
            for y in 0..=2 {
                let entity = world.create(dynec::comps![ Container =>
                    container::MaxVolume{volume: Volume{quantity: 10.}},
                    container::MaxPressure{pressure: Pressure { quantity: 10. }},
                ]);
                world.get_global::<Entities>().containers.insert((x, y), entity);
            }
        }
    }
}
