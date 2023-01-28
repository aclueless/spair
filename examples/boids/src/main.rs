use settings::Settings;
use simulation::Simulation;
use slider::{Slider, SliderProps};
use spair::prelude::*;

mod boid;
mod math;
mod settings;
mod simulation;
mod slider;

pub struct App {
    settings: Settings,
    generation: usize,
    paused: bool,
    simulation: spair::ChildComp<Simulation>,
}
impl App {
    fn reset_settings(&mut self) {
        self.settings = Settings::default();
        Settings::remove();
        self.update_simulation_settings();
    }

    fn update_simulation_settings(&self) {
        self.settings.store();
        self.simulation
            .comp()
            .callback_arg_mut(Simulation::update_params)
            .call_or_queue((self.settings.clone(), self.generation, self.paused));
    }

    fn restart_simulation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
        self.update_simulation_settings();
    }

    fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        self.update_simulation_settings();
    }

    fn boids(&mut self, value: usize) {
        self.settings.boids = value;
        self.update_simulation_settings();
    }
}

macro_rules! make_slider_helper_methods {
    ($(
        $setting_prop_name:ident {
            $($slider_prop_name:ident = $value:literal)+
        }
    )+) => {
        impl App {
            $(
                fn $setting_prop_name(&mut self, value: f64) {
                    self.settings.$setting_prop_name = value;
                    self.update_simulation_settings();
                }
            )+
        }

        mod sliders {
            use spair::prelude::*;
            use super::{App, Slider, SliderProps};

            $(
                pub fn $setting_prop_name(id: usize)
                    -> impl FnOnce(&App, &spair::Comp<App>) -> spair::Child<App, Slider, f64>
                {
                    move |_parent_state: &App, parent_comp: &spair::Comp<App>| {
                        Slider::with_props(
                            SliderProps::new(
                                id,
                                parent_comp.callback_arg_mut(App::$setting_prop_name),
                            )
                            $(
                                .$slider_prop_name($value)
                            )+
                        )
                        .with_updater(
                            |parent_state: &App| parent_state.settings.$setting_prop_name,
                            Slider::update_value
                        )
                    }
                }
            )+
        }
    };
}

make_slider_helper_methods! {
    visible_range {
        label="View Distance"
        max=500.0
        step=10.0
    }
    min_distance {
        label="Spacing"
        max=100.0
    }
    max_speed {
        label="Max Speed"
        max=50.0
    }
    cohesion_factor {
        label="Cohesion"
        max=0.5
        percentage=true
    }
    separation_factor {
        label="Separation"
        max=1.0
        percentage=true
    }
    alignment_factor {
        label="Alignment"
        max=0.5
        percentage=true
    }
    turn_speed_ratio {
        label="Turn Speed"
        max=1.5
        percentage=true
    }
    color_adapt_factor {
        label="Color Adaption"
        max=1.5
        percentage=true
    }
}

impl spair::Component for App {
    type Routes = ();
    fn render(&self, element: spair::Element<Self>) {
        element
            .h1(|h| h.class("title").rupdate("Boids").done())
            .component_ref(self.simulation.component_ref())
            .rupdate(Panel);
    }
}
struct Panel;
impl spair::Render<App> for Panel {
    fn render(self, nodes: spair::Nodes<App>) {
        let comp = nodes.comp();
        let state = nodes.state();
        nodes.div(|d| {
            d.class("panel").rupdate(SettingsPanel).div(|d| {
                d.class("panel__buttons")
                    .button(|b| {
                        let pause_text = if state.paused { "Resume" } else { "Pause" };
                        b.on_click(comp.handler_mut(App::toggle_pause))
                            .rupdate(pause_text);
                    })
                    .button(|b| {
                        b.on_click(comp.handler_mut(App::reset_settings))
                            .rupdate("Use Defaults");
                    })
                    .button(|b| {
                        b.on_click(comp.handler_mut(App::restart_simulation))
                            .rupdate("Restart");
                    });
            });
        });
    }
}
struct SettingsPanel;
impl spair::Render<App> for SettingsPanel {
    fn render(self, nodes: spair::Nodes<App>) {
        let mut id = 0;
        let mut next_id = || {
            id += 1;
            id
        };
        nodes.div(|d| {
            d.class("settings")
                .component_owned(|_pstate, pcomp| {
                    Slider::with_props(
                        SliderProps::new(
                            next_id(),
                            pcomp.callback_arg_mut(|state, value| state.boids(value as usize)),
                        )
                        .label("Number of Boids")
                        .min(1.0)
                        .max(600.0),
                    )
                    .with_updater(
                        |parent_state: &App| parent_state.settings.boids as f64,
                        Slider::update_value,
                    )
                })
                .component_owned(sliders::visible_range(next_id()))
                .component_owned(sliders::min_distance(next_id()))
                .component_owned(sliders::max_speed(next_id()))
                .component_owned(sliders::cohesion_factor(next_id()))
                .component_owned(sliders::separation_factor(next_id()))
                .component_owned(sliders::alignment_factor(next_id()))
                .component_owned(sliders::turn_speed_ratio(next_id()))
                .component_owned(sliders::color_adapt_factor(next_id()));
        });
    }
}

impl spair::Application for App {
    fn init(_comp: &spair::Comp<Self>) -> Self {
        let settings = Settings::load();
        Self {
            settings: settings.clone(),
            generation: 0,
            paused: false,
            simulation: Simulation::with_props(settings),
        }
    }
}

fn main() {
    App::mount_to_body()
}
