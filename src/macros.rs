#[macro_export]
macro_rules! set_arm {
    ( $match_if:ident ) => {
        $match_if.render_on_arm_index(line!())
    };
}
