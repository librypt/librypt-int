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
    let bits: u32 = parse_macro_input!(arg as LitInt).base10_parse().unwrap();
    assert!(
        bits % 8 == 0 && bits != 0,
        "Bits need to be 8 aligned and non zero"
    );
    let input = parse_macro_input!(input as ItemStruct);

    if let Fields::Unit = input.fields {
    } else {
        panic!("Expected unit struct");
    }

    let mut punct = Punctuated::new();

    let mut chunks = vec![];
    {
        let mut bits = bits;
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

    let last_member = Member::Unnamed(Index {
        index: chunks.len() as u32 - 1,
        span: Span::call_site(),
    });
    let last_member_type = types.first().unwrap().clone();

    let add_quote = quote! {
        let mut ret = #name::MIN;
        let mut carry = false;
        let mut other_carry = false;

        #(
            (ret.#idxs, carry) = self.#idxs.overflowing_add(
                carry as #types + other_carry as #types
            );
            (ret.#idxs, other_carry) = ret.#idxs.overflowing_add(rhs.#idxs);
        )*

        if carry || other_carry {
            ret = #name::MIN;
            ret.#last_member = carry as #last_member_type + other_carry as #last_member_type;
        }

        (ret, carry || other_carry)
    };

    let sub_quote = quote! {
        let mut ret = #name::MIN;
        let mut carry = false;
        let mut other_carry = false;

        #(
            (ret.#idxs, carry) = self.#idxs.overflowing_sub(
                carry as #types + other_carry as #types
            );
            (ret.#idxs, other_carry) = ret.#idxs.overflowing_sub(rhs.#idxs);
        )*

        if carry || other_carry {
            ret = #name::MAX;
            ret.#last_member -= carry as #last_member_type + other_carry as #last_member_type;
        }

        (ret, carry || other_carry)
    };

    let deftypes: Vec<_> = types.iter().rev().collect();

    quote! {
        #input

        impl #name {
            const MIN: #name = #name(#(#deftypes::MIN),*);
            const MAX: #name = #name(#(#deftypes::MAX),*);
            const BITS: u32 = #bits;

            pub const fn overflowing_add(self, rhs: #name) -> (#name, bool) {
               #add_quote
            }

            pub const fn overflowing_sub(self, rhs: #name) -> (#name, bool) {
                #sub_quote
            }
        }

        impl std::ops::Add for #name {
            type Output = #name;

            #[inline]
            fn add(self, other: #name) -> #name {
                let (ret, carry) = self.overflowing_add(other);

                debug_assert!(!carry, "attempt to add with overflow");

                ret
            }
        }

        impl std::ops::AddAssign for #name {
            #[inline]
            fn add_assign(&mut self, other: #name) {
                *self = *self + other;
            }
        }

        impl std::ops::Sub for #name {
            type Output = #name;

            #[inline]
            fn sub(self, other: #name) -> #name {
                let (ret, carry) = self.overflowing_sub(other);

                debug_assert!(!carry, "attempt to subtract with overflow");

                ret
            }
        }

        impl std::ops::SubAssign for #name {
            #[inline]
            fn sub_assign(&mut self, other: #name) {
                *self = *self - other;
            }
        }

        impl std::ops::Add<&#name> for &#name {
            type Output = #name;

            #[inline]
            fn add(self, other: &#name) -> #name {
                *self + *other
            }
        }


        impl std::ops::Sub<&#name> for &#name {
            type Output = #name;

            #[inline]
            fn sub(self, other: &#name) -> #name {
                *self - *other
            }
        }


        impl std::ops::Add<&#name> for #name {
            type Output = #name;

            #[inline]
            fn add(self, other: &#name) -> #name {
                self + *other
            }
        }

        impl std::ops::AddAssign<&#name> for #name {
            #[inline]
            fn add_assign(&mut self, other: &#name) {
                *self = *self + other;
            }
        }

        impl std::ops::Sub<&#name> for #name {
            type Output = #name;

            #[inline]
            fn sub(self, other: &#name) -> #name {
                self - *other
            }
        }

        impl std::ops::SubAssign<&#name> for #name {
            #[inline]
            fn sub_assign(&mut self, other: &#name) {
                *self = *self - other;
            }
        }

        impl<'a> std::ops::Add<#name> for &'a #name {
            type Output = #name;

            #[inline]
            fn add(self, other: #name) -> #name {
                *self + other
            }
        }

        impl<'a> std::ops::Sub<#name> for &'a #name {
            type Output = #name;

            #[inline]
            fn sub(self, other: #name) -> #name {
                *self - other
            }
        }

        impl Default for #name {
            #[inline]
            fn default() -> #name {
                #name::MIN
            }
        }

        impl From<u8> for #name {
            #[inline]
            fn from(value: u8) -> #name {
                let mut ret = #name::MIN;
                ret.#last_member = value as #last_member_type;
                ret
            }
        }

        impl From<u16> for #name {
            #[inline]
            fn from(value: u16) -> #name {
                let mut ret = #name::MIN;
                ret.#last_member = value as #last_member_type;
                ret
            }
        }

        impl From<u32> for #name {
            #[inline]
            fn from(value: u32) -> #name {
                let mut ret = #name::MIN;
                ret.#last_member = value as #last_member_type;
                ret
            }
        }

        impl From<u64> for #name {
            #[inline]
            fn from(value: u64) -> #name {
                let mut ret = #name::MIN;
                ret.#last_member = value as #last_member_type;
                ret
            }
        }

        impl From<u128> for #name {
            #[inline]
            fn from(value: u128) -> #name {
                let mut ret = #name::MIN;
                ret.#last_member = value as #last_member_type;
                ret
            }
        }

        impl From<i8> for #name {
            #[inline]
            fn from(value: i8) -> #name {
                (value as u128).into()
            }
        }

        impl From<i16> for #name {
            #[inline]
            fn from(value: i16) -> #name {
                (value as u128).into()
            }
        }

        impl From<i32> for #name {
            #[inline]
            fn from(value: i32) -> #name {
                (value as u128).into()
            }
        }

        impl From<i64> for #name {
            #[inline]
            fn from(value: i64) -> #name {
                (value as u128).into()
            }
        }

        impl From<i128> for #name {
            #[inline]
            fn from(value: i128) -> #name {
                (value as u128).into()
            }
        }

        impl From<#name> for u8 {
            #[inline]
            fn from(value: #name) -> u8 {
                value.#last_member as u8
            }
        }

        impl From<#name> for u16 {
            #[inline]
            fn from(value: #name) -> u16 {
                value.#last_member as u16
            }
        }

        impl From<#name> for u32 {
            #[inline]
            fn from(value: #name) -> u32 {
                value.#last_member as u32
            }
        }

        impl From<#name> for u64 {
            #[inline]
            fn from(value: #name) -> u64 {
                value.#last_member as u64
            }
        }

        impl From<#name> for u128 {
            #[inline]
            fn from(value: #name) -> u128 {
                value.#last_member as u128
            }
        }

        impl From<#name> for i8 {
            #[inline]
            fn from(value: #name) -> i8 {
                u128::from(value) as i8
            }
        }

        impl From<#name> for i16 {
            #[inline]
            fn from(value: #name) -> i16 {
                u128::from(value) as i16
            }
        }

        impl From<#name> for i32 {
            #[inline]
            fn from(value: #name) -> i32 {
                u128::from(value) as i32
            }
        }

        impl From<#name> for i64 {
            #[inline]
            fn from(value: #name) -> i64 {
                u128::from(value) as i64
            }
        }

        impl From<#name> for i128 {
            #[inline]
            fn from(value: #name) -> i128 {
                u128::from(value) as i128
            }
        }
    }
    .into()
}
