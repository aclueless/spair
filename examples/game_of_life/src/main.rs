use gloo_timers::callback::Interval;
use rand::Rng;
use spair::{
    CallbackArg, Context,
    prelude::{create_view, impl_component},
    web_sys::MouseEvent,
};

use crate::cell::Cellule;

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
            if rand::rng().random() {
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

#[impl_component]
impl App {
    fn create(cc: &Context<Self>) {}
    fn update(uc: &Context<Self>) {}
    fn view() {
        div(
            replace_at_element_id = "root",
            section(
                class = "game-container",
                header(
                    class = "app-header",
                    img(
                        class = "app-logo",
                        alt = "Game of Life logo",
                        src = "favicon.ico",
                    ),
                    h1(class = "app-title", "Game of Life"),
                ),
                section(class = "game-area", GameBoard().update(uc), Buttons(cc)),
            ),
            footer(
                class = "app-footer",
                strong(
                    class = "footer-text",
                    "Game of Life - a port from Yew's implementation",
                ),
                ", ",
                a(href_str = "https://github.com/aclueless/spair", "Source"),
                ", ",
                a(
                    href_str = "https://github.com/yewstack/yew",
                    "Original source",
                ),
            ),
        )
    }
}

#[create_view]
impl GameBoard {
    fn create() {}
    fn update(uc: &Context<App>) {}
    fn view() {
        div(
            class = "game-of-life",
            spair_list(
                uc.state
                    .cellules
                    .chunks(uc.state.cellules_width)
                    .enumerate(),
                |row| -> &usize { &row.0 },
                |_crow| {},
                |(index, urow)| {
                    let offset = index * uc.state.cellules_width;
                },
                div(
                    class = "game-row",
                    spair_list(
                        urow.iter().enumerate(),
                        |cell| -> &usize { &cell.0 },
                        |(index, _ccell)| {
                            let cc = uc;
                            let index = offset + index;
                        },
                        |ucell| {},
                        div(
                            class = "game-cellule",
                            class_or = (ucell.1.is_alive(), "cellule-live", "cellule-dead"),
                            on_click = cc
                                .comp
                                .callback_arg(move |state, _| state.toggle_cellule(index)),
                        ),
                    ),
                ),
            ),
        )
    }
}

#[create_view]
impl Buttons {
    fn create(cc: &Context<App>) {}
    fn update() {}
    fn view() {
        div(
            class = "game-buttons",
            Button("Random", cc.comp.callback_arg(|state, _| state.random())),
            Button("Step", cc.comp.callback_arg(|state, _| state.step())),
            Button("Start", cc.comp.callback_arg(|state, _| state.start())),
            Button("Stop", cc.comp.callback_arg(|state, _| state.stop())),
            Button("Reset", cc.comp.callback_arg(|state, _| state.reset())),
        )
    }
}

#[create_view]
impl Button {
    fn create(name: &str, callback: CallbackArg<MouseEvent>) {}
    fn update() {}
    fn view() {
        button(class = "game-button", on_click = callback, name)
    }
}

fn main() {
    spair::start_app(|comp| {
        let callback = comp.callback(App::tick);
        let interval = Interval::new(200, move || callback.call());

        let (cellules_width, cellules_height) = (53, 40);
        App {
            active: false,
            cellules: vec![Cellule::new_dead(); cellules_width * cellules_height],
            cellules_width,
            cellules_height,
            _interval: interval,
        }
    });
}
