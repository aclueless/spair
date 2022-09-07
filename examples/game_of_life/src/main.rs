use cell::Cellule;
use gloo::timers::callback::Interval;
use rand::Rng;
use spair::prelude::*;

mod cell;

pub struct App {
    active: bool,
    cellules: Vec<Cellule>,
    cellules_width: usize,
    cellules_height: usize,
    _interval: Interval,
}

impl App {
    pub fn random_mutate(&mut self) {
        for cellule in self.cellules.iter_mut() {
            if rand::thread_rng().gen() {
                cellule.set_alive();
            } else {
                cellule.set_dead();
            }
        }
    }

    fn reset_world(&mut self) {
        for cellule in self.cellules.iter_mut() {
            cellule.set_dead();
        }
    }

    fn step(&mut self) {
        let mut to_dead = Vec::new();
        let mut to_live = Vec::new();
        for row in 0..self.cellules_height {
            for col in 0..self.cellules_width {
                let neighbors = self.neighbors(row as isize, col as isize);

                let current_idx = self.row_col_as_idx(row as isize, col as isize);
                if self.cellules[current_idx].is_alive() {
                    if Cellule::alone(&neighbors) || Cellule::overpopulated(&neighbors) {
                        to_dead.push(current_idx);
                    }
                } else if Cellule::can_be_revived(&neighbors) {
                    to_live.push(current_idx);
                }
            }
        }
        to_dead
            .iter()
            .for_each(|idx| self.cellules[*idx].set_dead());
        to_live
            .iter()
            .for_each(|idx| self.cellules[*idx].set_alive());
    }

    fn neighbors(&self, row: isize, col: isize) -> [Cellule; 8] {
        [
            self.cellules[self.row_col_as_idx(row + 1, col)],
            self.cellules[self.row_col_as_idx(row + 1, col + 1)],
            self.cellules[self.row_col_as_idx(row + 1, col - 1)],
            self.cellules[self.row_col_as_idx(row - 1, col)],
            self.cellules[self.row_col_as_idx(row - 1, col + 1)],
            self.cellules[self.row_col_as_idx(row - 1, col - 1)],
            self.cellules[self.row_col_as_idx(row, col - 1)],
            self.cellules[self.row_col_as_idx(row, col + 1)],
        ]
    }

    fn row_col_as_idx(&self, row: isize, col: isize) -> usize {
        let row = wrap(row, self.cellules_height as isize);
        let col = wrap(col, self.cellules_width as isize);

        row * self.cellules_width + col
    }

    fn random(&mut self) {
        self.random_mutate();
        log::info!("Random");
    }

    fn start(&mut self) -> spair::ShouldRender {
        self.active = true;
        log::info!("Start");
        spair::ShouldRender::No
    }

    fn reset(&mut self) {
        self.reset_world();
        log::info!("Reset");
    }

    fn stop(&mut self) -> spair::ShouldRender {
        self.active = false;
        log::info!("Stop");
        spair::ShouldRender::No
    }

    fn toggle_cellule(&mut self, index: usize) {
        let cellule = self.cellules.get_mut(index).unwrap();
        cellule.toggle();
    }

    fn tick(&mut self) -> spair::ShouldRender {
        if self.active {
            self.step();
            spair::ShouldRender::Yes
        } else {
            spair::ShouldRender::No
        }
    }
}
impl spair::Component for App {
    type Routes = ();

    fn render(&self, element: spair::Element<Self>) {
        let comp = element.comp();
        element.div(|d| {
            d.section(|s| {
                s.class("game-container")
                    .header(|h| {
                        h.class("app-header")
                            .img(|i| {
                                i.alt("The app logo").src("favicon.ico").class("app-logo");
                            })
                            .h1(|h| {
                                h.class("app-title").rupdate("Game of Life");
                            });
                    })
                    .section(|s| {
                        s.class("game-area")
                            .div(|d| {
                                d.class("game-of-life").keyed_list_clone(
                                    self.cellules
                                        .chunks(self.cellules_width)
                                        .enumerate()
                                        .map(|r| Row(r.0, r.1)),
                                );
                            })
                            .div(|d| {
                                d.class("game-buttons")
                                    .rfn(|ns| button(ns, "Random", comp.handler_mut(App::random)))
                                    .rfn(|ns| button(ns, "Step", comp.handler_mut(App::step)))
                                    .rfn(|ns| button(ns, "Start", comp.handler_mut(App::start)))
                                    .rfn(|ns| button(ns, "Stop", comp.handler_mut(App::stop)))
                                    .rfn(|ns| button(ns, "Reset", comp.handler_mut(App::reset)));
                            });
                    });
            })
            .footer(|f| {
                f.class("app-footer")
                    .strong(|s| {
                        s.class("footer-text")
                            .rupdate("Game of Life - a port from Yew's implementation");
                    })
                    .a(|a| {
                        a.href_str("https://github.com/yewstack/yew")
                            .target(spair::Target::_Blank)
                            .rupdate("source");
                    });
            });
        });
    }
}

fn button(nodes: spair::Nodes<App>, name: &str, h: impl spair::Click) {
    nodes.button(|b| {
        b.class("game-button").on_click(h).rupdate(name);
    });
}

#[derive(Clone, Copy)]
struct Row<'a>(usize, &'a [Cellule]);
impl<'k, 'a> spair::Keyed<'k> for Row<'a> {
    type Key = u32;
    fn key(&self) -> u32 {
        self.0 as u32
    }
}

impl<'a> spair::ListItemRender<App> for Row<'a> {
    const ROOT_ELEMENT_TAG: &'static str = "div";
    fn render(&self, element: spair::Element<App>) {
        let comp = element.comp();
        let offset = self.0 * element.state().cellules_width;
        element.class("game-row").klwr_clone(
            self.1.iter().enumerate(),
            "div",
            |&(index, _)| index as u32,
            |&(index, cellule), div| {
                let index = offset + index;
                div.class("game-cellule")
                    .class_or(cellule.is_alive(), "cellule-live", "cellule-dead")
                    .on_click(comp.handler_mut(move |state| state.toggle_cellule(index)));
            },
        );
    }
}

fn wrap(coord: isize, range: isize) -> usize {
    let result = if coord < 0 {
        coord + range
    } else if coord >= range {
        coord - range
    } else {
        coord
    };
    result as usize
}

impl spair::Application for App {
    fn init(comp: &spair::Comp<Self>) -> Self {
        let callback = comp.callback_mut(App::tick);
        let interval = Interval::new(200, move || callback.emit());

        let (cellules_width, cellules_height) = (53, 40);

        Self {
            active: false,
            cellules: vec![Cellule::new_dead(); cellules_width * cellules_height],
            cellules_width,
            cellules_height,
            _interval: interval,
        }
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::trace!("Initializing spair...");
    App::mount_to_body();
}
