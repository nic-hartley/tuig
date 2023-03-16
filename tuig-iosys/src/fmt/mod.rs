//! Implements CLI-compatible text formatting.
//! 
//! This module defaults to a 'lowest common subset' across all render targets. Additional functionality is enabled if
//! explicitly turned on, but be aware that incompatible render hardware will simply ignore it. To achieve "good" UI
//! across a variety of render targets, you'll need to write your own code taking maximum advantage of avaliable
//! features and degrading nicely, for your set of supported backends. See the backends' documentation for supported
//! feature sets.
//! 
//! This entire module is `#![no_std]` compatible. See the crate root for more info.
//! 
//! - By default:
//!   - 16 basic [`Color`]s (blue, green, cyan, red, magenta, yellow, black, and the bright equivalents)
//!   - Setting foreground and background