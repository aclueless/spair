pub struct SvgStaticAttributes<'a,C>(super::SvgUpdater<'a,C>);

impl<'a, C: crate::component::Component> SvgStaticAttributes<'a, C> {
    pub(super) fn new(su: super::SvgUpdater<'a, C>) -> Self {
        Self(su)
    }

    /// Use this method when the compiler complains about expected `()` but found something else and you don't want to add a `;`
    pub fn done(self) {}
}

pub trait SvgAttributeSetter {
    //
}
