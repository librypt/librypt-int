use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Paren, Field, Fields, FieldsUnnamed,
    ItemStruct, LitInt, Path, PathArguments, PathSegment, Type, TypePath, Visibility,
};

use proc_macro2::{Ident, Span};

#[proc_macro_attribute]
pub fn bituint(arg: TokenStream, input: TokenStream) -> TokenStream {
    let mut bits: usize = parse_macro_input!(arg as LitInt).base10_parse().unwrap();
    assert!(bits % 8 == 0, "Bits needs to be 8 aligned");
    let mut input = parse_macro_input!(input as ItemStruct);

    if let Fields::Unit = input.fields {
    } else {
        panic!("Expected unit struct");
    }

    let mut punct = Punctuated::new();

    while bits != 0 {
        let intbits = if bits >= 128 {
            128
        } else if bits >= 64 {
            64
        } else if bits >= 32 {
            32
        } else if bits >= 16 {
            16
        } else {
            8
        };

        bits -= intbits;
        let mut segments = Punctuated::new();
        segments.push(PathSegment {
            ident: Ident::new(format!("u{}", intbits).as_str(), Span::call_site()),
            arguments: PathArguments::None,
        });

        punct.push(Field {
            attrs: vec![],
            vis: Visibility::Inherited,
            ident: None,
            colon_token: None,
            ty: Type::Path(TypePath {
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments,
                },
            }),
        });
    }

    let input = ItemStruct {
        fields: Fields::Unnamed(FieldsUnnamed {
            paren_token: Paren {
                span: Span::call_site(),
            },
            unnamed: punct,
        }),
        ..input
    };

    quote! {
        #input
    }
    .into()
}
