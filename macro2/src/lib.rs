use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use proc_macro_error::proc_macro_error;
use std::result::Result as StdResult;
use syn::{parse_macro_input, AttributeArgs, Item};

mod class;
use class::{impl_enum, impl_struct};

mod method;

enum Error {
    Syn(syn::Error),
    Darling(darling::Error),
}

impl Error {
    pub fn into_stream(self) -> TokenStream {
        match self {
            Self::Syn(e) => e.into_compile_error(),
            Self::Darling(e) => e.write_errors(),
        }
    }
}

type Result<T> = StdResult<T, Error>;

impl From<syn::Error> for Error {
    fn from(value: syn::Error) -> Self {
        Self::Syn(value)
    }
}

impl From<darling::Error> for Error {
    fn from(value: darling::Error) -> Self {
        Self::Darling(value)
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn jsclass(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let attr: AttributeArgs = parse_macro_input!(attr);
    let item = parse_macro_input!(item as Item);
    match item {
        Item::Struct(struct_) => impl_struct(attr, struct_)
            .unwrap_or_else(Error::into_stream)
            .into(),
        Item::Enum(enum_) => impl_enum(attr, enum_)
            .unwrap_or_else(Error::into_stream)
            .into(),
        unsupported => syn::Error::new_spanned(
            unsupported,
            "#[jsclass] can only be used on `struct` declarations.",
        )
        .into_compile_error()
        .into(),
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn jsmethods(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let item = parse_macro_input!(item as Item);
    match item {
        Item::Impl(item) => {
            if item.trait_.is_some() {
                return syn::Error::new_spanned(
                    item,
                    "#[jsmethods] cannot be used on trait implementations.",
                )
                .into_compile_error()
                .into();
            }
            method::expand(item)
                .unwrap_or_else(Error::into_stream)
                .into()
        }
        unsupported => syn::Error::new_spanned(
            unsupported,
            "#[jsmethods] can only be used on `impl` declarations.",
        )
        .into_compile_error()
        .into(),
    }
}
