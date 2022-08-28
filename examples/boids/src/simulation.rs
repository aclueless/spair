use gloo::timers::callback::Interval;
use spair::prelude::*;

use crate::boid::Boid;
use crate::math::Vector2D;
use crate::settings::Settings;

pub const SIZE: Vector2D = Vector2D::new(1600.0, 1000.0);

#[derive(Clone, Debug, PartialEq)]
pub struct Props {
    pub settings: Settings,
    pub generation: usize,
    pub paused: bool,
}

pub struct Simulation {
    comp: spair::Comp<Self>,
    boids: Vec<Boid>,
    interval: Interval,
    settings: Settings,
    generation: usize,
    paused: bool,
}

impl Simulation {
    fn new(settings: Settings, comp: &spair::Comp<Self>) -> Self {
        let boids = (0..settings.boids)
            .map(|_| Boid::new_random(&settings))
            .collect();

        let interval = {
            let cb = comp.callback_mut(Self::tick);
            Interval::new(settings.tick_interval_ms as u32, move || {
                cb.call();
            })
        };

        Self {
            comp: comp.clone(),
            boids,
            interval,
            settings,
            generation: 0,
            paused: false,
        }
    }

    fn tick(&mut self) -> spair::ShouldRender {
        if self.paused {
            spair::ShouldRender::No
        } else {
            Boid::update_all(&self.settings, &mut self.boids);
            spair::ShouldRender::Yes
        }
    }

    pub fn update_params(
        &mut self,
        (settings, generation, paused): (Settings, usize, bool),
    ) -> spair::ShouldRender {
        self.paused = paused;
        let should_reset = self.settings != settings || self.generation != generation;
        self.settings = settings;
        self.generation = generation;
        if should_reset {
            self.boids.clear();

            let settings = &self.settings;
            self.boids
                .resize_with(settings.boids, || Boid::new_random(settings));

            // as soon as the previous task is dropped it is cancelled.
            // We don't need to worry about manually stopping it.
            self.interval = {
                let cb = self.comp.callback_mut(Self::tick);
                Interval::new(settings.tick_interval_ms as u32, move || {
                    cb.call();
                })
            };
            spair::ShouldRender::Yes
        } else {
            spair::ShouldRender::No
        }
    }
}

impl spair::Component for Simulation {
    type Routes = ();
    fn render(&self, element: spair::Element<Self>) {
        let view_box = format!("0 0 {} {}", SIZE.x, SIZE.y);
        element
            .as_svg_element()
            .class("simulation-window")
            .view_box(view_box)
            .list_clone(self.boids.iter());
    }
}

impl spair::AsChildComp for Simulation {
    const ROOT_ELEMENT_TAG: spair::TagName = spair::TagName::Svg(spair::SvgTag("svg"));
    type Properties = Settings;
    fn init(comp: &spair::Comp<Self>, props: Self::Properties) -> Self {
        Self::new(props, comp)
    }
}
