#[macro_use]
mod macros;

pub mod base;
pub mod html;
pub mod svg;

#[derive(Copy, Clone)]
pub enum ListElementCreation {
    Clone,
    New,
}

impl ListElementCreation {
    pub fn use_template(&self) -> bool {
        match self {
            Self::Clone => true,
            Self::New => false,
        }
    }
}
