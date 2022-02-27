//! # `calypso_filety`
//!
//! Binary file type definitions, parsers, high-level interfaces, and more for
//! file types used in [Calypso](https://github.com/calypso-lang/calypso).
//!
//! File types currently included are
//! - Calypso Container File Format (CCFF). For more information, see
//!   [the "spec"](https://github.com/calypso-lang/calypso/blob/main/libs/calypso_filety/ccff.md).
//!   The interface to this format is implemented in [`ccff`].
#![doc(html_root_url = "https://calypso-lang.github.io/rustdoc/calypso_filety/index.html")]
#![warn(clippy::pedantic)]

/// Calypso Container File Format. See the [module-level docs](self) for more information.
pub mod ccff;
