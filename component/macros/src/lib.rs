#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(clippy::shadow_unrelated)]
#![allow(clippy::doc_markdown)]
#![allow(unstable_name_collisions)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::let_underscore_drop)]
#![allow(clippy::too_many_lines)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod parse_frames;
mod utils;

#[proc_macro_derive(ParseFrames2, attributes(default))]
pub fn derive_parse_frames(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    parse_frames::do_derive(&ast).into()
}
