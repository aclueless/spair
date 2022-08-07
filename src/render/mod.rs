#[macro_use]
mod macros;

pub mod base;
pub mod html;
#[cfg(feature = "svg")]
pub mod svg;

pub struct SeeDeprecationNoteOrMethodDocForInformation;

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
