/// Short syntax for feature-gated function calls
macro_rules! feature_switch {
    ( $( $feature:literal => $call:expr ),* $(,)? ) => { loop {
        $(
            #[cfg(feature = $feature)]
            {
                break $call;
            }
        )*
        unreachable!("feature_switch! but no features enabled!");
    } }
}

pub(crate) use feature_switch;
