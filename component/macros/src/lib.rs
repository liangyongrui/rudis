#![allow(clippy::panic)]

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod parse_frames;
mod utils;

#[proc_macro_derive(ParseFrames, attributes(default, optional))]
pub fn derive_parse_frames(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    parse_frames::do_derive(&ast).into()
}
