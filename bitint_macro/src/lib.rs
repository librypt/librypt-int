use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Paren, Field, Fields, FieldsUnnamed, Index,
    ItemStruct, LitInt, Member, Path, PathArguments, PathSegment, Token, Type, TypePath, VisPublic,
    Visibility,
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

    let mut chunks = vec![];
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
        chunks.push(intbits);
        let mut segments = Punctuated::new();
        segments.push(PathSegment {
            ident: Ident::new(format!("u{}", intbits).as_str(), Span::call_site()),
            arguments: PathArguments::None,
        });

        punct.push(Field {
            attrs: vec![],
            vis: Visibility::Public(VisPublic {
                pub_token: Token![pub](Span::call_site()),
            }),
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

    chunks.reverse();

    let input = ItemStruct {
        fields: Fields::Unnamed(FieldsUnnamed {
            paren_token: Paren {
                span: Span::call_site(),
            },
            unnamed: punct,
        }),
        ..input
    };

    let name = input.ident.clone();

    let add_quote = {
        let idxs: Vec<_> = chunks
            .iter()
            .enumerate()
            .rev()
            .map(|(idx, _)| {
                Member::Unnamed(Index {
                    index: idx as u32,
                    span: Span::call_site(),
                })
            })
            .collect();

        let types: Vec<_> = chunks
            .iter()
            .map(|x| Ident::new(format!("u{}", x).to_string().as_str(), Span::call_site()))
            .collect();

        let deftypes: Vec<_> = types.iter().rev().collect();

        quote! {
            let mut ret = Self(#(0 as #deftypes),*);
            let mut carry = false;
            let mut other_carry = false;

            #(
                (ret.#idxs, carry) = self.#idxs.overflowing_add(
                    carry as #types + other_carry as #types
                );
                (ret.#idxs, other_carry) = ret.#idxs.overflowing_add(other.#idxs);
            )*

            assert!(!carry && !other_carry);

            ret
        }
    };

    let sub_quote = {
        let idxs: Vec<_> = chunks
            .iter()
            .enumerate()
            .rev()
            .map(|(idx, _)| {
                Member::Unnamed(Index {
                    index: idx as u32,
                    span: Span::call_site(),
                })
            })
            .collect();

        let types: Vec<_> = chunks
            .iter()
            .map(|x| Ident::new(format!("u{}", x).to_string().as_str(), Span::call_site()))
            .collect();

        let deftypes: Vec<_> = types.iter().rev().collect();

        quote! {
            let mut ret = Self(#(0 as #deftypes),*);
            let mut carry = false;
            let mut other_carry = false;

            #(
                (ret.#idxs, carry) = self.#idxs.overflowing_sub(
                    carry as #types + other_carry as #types
                );
                (ret.#idxs, other_carry) = ret.#idxs.overflowing_sub(other.#idxs);
            )*

            assert!(!carry && !other_carry);

            ret
        }
    };

    quote! {
        #input

        impl std::ops::Add for #name {
            type Output = Self;

            fn add(self, other: Self) -> Self {
                #add_quote
            }
        }

        impl std::ops::Sub for #name {
            type Output = Self;

            fn sub(self, other: Self) -> Self {
                #sub_quote
            }
        }
    }
    .into()
}
