#![no_std]
#![recursion_limit = "128"]

extern crate alloc;
extern crate proc_macro;

use core::panic;

use alloc::{
    format,
    string::{String, ToString},
};
use proc_macro::TokenStream;
use proc_macro_roids::namespace_parameters;
use quote::{format_ident, quote};
use syn::{
    parse_quote, Data, DataEnum, DeriveInput, Fields, FieldsUnnamed, Lit, Meta, NestedMeta, Path,
    Variant,
};

/// Attributes that should be copied across.

/// Derives a struct for each enum variant.
///
/// Struct fields including their attributes are copied over.
#[cfg(not(tarpaulin_include))]
#[proc_macro_derive(HoloUcgEffect, attributes(holo_ucg))]
pub fn enum_variant_type(input: TokenStream) -> TokenStream {
    use syn::parse_macro_input;

    let ast = parse_macro_input!(input as DeriveInput);

    // Need to do this, otherwise we can't unit test the input.
    ser_de_token_for_enum_impl(ast).into()
}

#[inline]
fn ser_de_token_for_enum_impl(ast: DeriveInput) -> proc_macro2::TokenStream {
    let enum_name = &ast.ident;
    let data_enum = data_enum(&ast);
    let variants = &data_enum.variants;

    let mut ser_de_token_for_enum = proc_macro2::TokenStream::new();

    let ns: Path = parse_quote!(holo_ucg);

    // serialize effect tokens
    let ser_variants_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let (token, infix, transparent) = attributes(variant, &ns);
        let variant_fields = &variant.fields;

        let variant_capture = match variant_fields {
            Fields::Unit => quote! {},
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let ns = (0..unnamed.len()).map(|n| format_ident!("_{}", n));
                quote! { (#(#ns,)*) }
            }
            a => panic!("{:?}", a),
        };
        let variant_into = match variant_fields {
            Fields::Unit => quote! { #token.into() },
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let mut ns = (0..unnamed.len()).map(|n| {
                    let i = format_ident!("_{}", n);
                    quote! { #i.into() }
                });
                if transparent {
                    quote! { [#(#ns,)*].into() }
                } else if let Some(infix) = infix {
                    let first = ns.next();
                    quote! { [#first, #infix.into(), #(#ns,)*].into() }
                } else {
                    quote! { [#token.into(), #(#ns,)*].into() }
                }
            }
            a => panic!("{:?}", a),
        };

        quote! {
            #enum_name::#variant_name #variant_capture => #variant_into,
        }
    });
    let impl_ser_for_enum = quote! {

        impl core::convert::From<#enum_name> for crate::card_effects::parse::Tokens {
            fn from(value: #enum_name) -> Self {
                match value {
                    #(#ser_variants_arms)*
                }
            }
        }
    };

    // deserialize effect tokens
    let de_variants_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let (token, _infix, transparent) = attributes(variant, &ns);
        let variant_fields = &variant.fields;

        if transparent {
            return quote! {};
        }

        let variant_take_params = match variant_fields {
            Fields::Unit => quote! {},
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let ns = (0..unnamed.len()).map(|_| quote! { tokens.take_param()? });
                quote! { (#(#ns,)*) }
            }
            a => panic!("{:?}", a),
        };

        quote! {
            #token => #enum_name::#variant_name #variant_take_params,
        }
    });
    let de_variants_transparent = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let (_token, _infix, transparent) = attributes(variant, &ns);
        let variant_fields = &variant.fields;

        if !transparent {
            return quote! {};
        }

        let variant_ok_capture = match variant_fields {
            Fields::Unit => {
                panic!("The `transparent` attribute argument cannot be used with unit enum.")
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let ns = (0..unnamed.len()).map(|n| format_ident!("_{}", n));
                quote! { (#(Ok(#ns),)*) }
            }
            a => panic!("{:?}", a),
        };
        let variant_take_params = match variant_fields {
            Fields::Unit => {
                panic!("The `transparent` attribute argument cannot be used with unit enum.")
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let ns = (0..unnamed.len()).map(|_| quote! { tokens.take_param() });
                quote! { (#(#ns,)*) }
            }
            a => panic!("{:?}", a),
        };
        let variant_capture = match variant_fields {
            Fields::Unit => {
                panic!("The `transparent` attribute argument cannot be used with unit enum.")
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let ns = (0..unnamed.len()).map(|n| format_ident!("_{}", n));
                quote! { (#(#ns,)*) }
            }
            a => panic!("{:?}", a),
        };

        quote! {
            if let #variant_ok_capture = #variant_take_params {
                return Ok(#enum_name::#variant_name #variant_capture);
            }
        }
    });

    // handle infix effects tokens
    let infix_tokens_inserts = variants.iter().map(|variant| {
        let (token, infix, _transparent) = attributes(variant, &ns);
        if infix.is_none() {
            return quote! {};
        }
        quote! {
            map.insert(#infix, #token);
        }
    });
    let infix_tokens_map = quote! {
        fn infix_token_map() -> &'static std::collections::HashMap<&'static str, &'static str> {
            use std::collections::HashMap;
            use std::sync::OnceLock;
            static INFIX_TOKEN_MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
            INFIX_TOKEN_MAP.get_or_init(|| {
                let mut map = HashMap::new();
                #(#infix_tokens_inserts)*
                map
            })
        }
    };

    let str_enum_name = format!("{enum_name}");
    let impl_de_for_enum = quote! {
        impl crate::card_effects::parse::ParseTokens for #enum_name {
            #infix_tokens_map

            fn parse_tokens(tokens: &mut std::collections::vec_deque::VecDeque<crate::card_effects::parse::Tokens>) -> std::result::Result<Self, crate::card_effects::error::Error> {
                use crate::card_effects::parse::TakeParam;
                Self::process_infix_tokens(tokens);
                let s = Self::take_string(tokens)?;
                #[allow(unreachable_code)]
                Ok(match s.as_str() {
                    #(#de_variants_arms)*
                    _ => {
                        Self::return_string(tokens, s.clone());
                        #(#de_variants_transparent)*
                        return Err(crate::card_effects::error::Error::UnexpectedToken(#str_enum_name.into(), s));
                    }
                })
            }
        }
    };

    // put it all together
    ser_de_token_for_enum.extend(impl_ser_for_enum);
    ser_de_token_for_enum.extend(impl_de_for_enum);

    ser_de_token_for_enum
}

fn data_enum(ast: &DeriveInput) -> &DataEnum {
    if let Data::Enum(data_enum) = &ast.data {
        data_enum
    } else {
        panic!("`EnumVariantType` derive can only be used on an enum.");
    }
}

fn attributes(variant: &Variant, ns: &Path) -> (String, Option<String>, bool) {
    let evt_meta_lists = namespace_parameters(&variant.attrs, ns);
    let mut token = variant.ident.to_string().to_lowercase();
    let mut infix = None;
    let mut transparent = false;
    for meta in evt_meta_lists {
        match meta {
            NestedMeta::Meta(Meta::NameValue(name_value)) => {
                if let (true, Lit::Str(lit_str)) =
                    (name_value.path.is_ident("token"), &name_value.lit)
                {
                    token = lit_str.value();
                } else if let (true, Lit::Str(lit_str)) =
                    (name_value.path.is_ident("infix"), &name_value.lit)
                {
                    infix = Some(lit_str.value());
                } else {
                    panic!("Expected `holo_ucg` attribute argument in the form: `#[holo_ucg(token = \"some_token\")]`");
                }
            }
            NestedMeta::Meta(Meta::Path(path)) => {
                if path.is_ident("transparent") {
                    transparent = true;
                } else {
                    panic!("Expected `holo_ucg` attribute argument in the form: `#[holo_ucg(transparent)]`");
                }
            }
            a => panic!("{:?}", a),
        }
    }
    (token, infix, transparent)
}

#[cfg(test)]
mod tests {
    extern crate alloc;

    use alloc::string::ToString;
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::{parse_quote, DeriveInput};

    use super::ser_de_token_for_enum_impl;

    #[test]
    fn generates_correct_tokens_for_basic_enum() {
        let ast: DeriveInput = parse_quote! {
            pub enum MyEnum {
                /// Unit variant.
                #[evt(derive(Clone, Copy, Debug, PartialEq))]
                #[holo_ucg(token = "unit", infix = "=")]
                Unit,
                /// Tuple variant.
                #[evt(derive(Debug))]
                #[holo_ucg(token = "tuple")]
                Tuple(u32, u64),
                /// Tuple variant.
                #[evt(derive(Debug))]
                #[holo_ucg(token = "plus", infix = "+")]
                TupleInfix(u32, u64),
                /// Tuple variant.
                #[evt(derive(Debug))]
                #[holo_ucg(transparent)]
                Transparent(u32, u32),
            }
        };

        let actual_tokens = ser_de_token_for_enum_impl(ast);
        let expected_tokens = quote! {
            impl core::convert::From<MyEnum> for crate::card_effects::parse::Tokens {
                fn from(value: MyEnum) -> Self {
                    match value {
                        MyEnum::Unit => "unit".into(),
                        MyEnum::Tuple(_0, _1,) => ["tuple".into(), _0.into(), _1.into(),].into(),
                        MyEnum::TupleInfix(_0, _1,) => [_0.into(), "+".into(),  _1.into(),].into(),
                        MyEnum::Transparent(_0, _1,) => [_0.into(), _1.into(),].into(),
                    }
                }
            }

            impl crate::card_effects::parse::ParseTokens for MyEnum {
                fn infix_token_map(
                ) -> &'static std::collections::HashMap<&'static str, &'static str>
                {
                    use std::collections::HashMap;
                    use std::sync::OnceLock;
                    static INFIX_TOKEN_MAP: OnceLock<HashMap<&'static str, &'static str>> =
                        OnceLock::new();
                    INFIX_TOKEN_MAP.get_or_init(|| {
                        let mut map = HashMap::new();
                        map.insert("=", "unit");
                        map.insert("+", "plus");
                        map
                    })
                }
                fn parse_tokens(
                    tokens: &mut std::collections::vec_deque::VecDeque<
                        crate::card_effects::parse::Tokens
                    >
                ) -> std::result::Result<Self, crate::card_effects::error::Error> {
                    use crate::card_effects::parse::TakeParam;
                    Self::process_infix_tokens(tokens);
                    let s = Self::take_string(tokens)?;
                    #[allow(unreachable_code)]
                    Ok(match s.as_str() {
                        "unit" => MyEnum::Unit,
                        "tuple" => MyEnum::Tuple(tokens.take_param()?, tokens.take_param()?,),
                        "plus" => MyEnum::TupleInfix(tokens.take_param()?, tokens.take_param()?,),
                        _ => {
                            Self::return_string(tokens, s.clone());
                            if let (Ok(_0), Ok(_1),) = (tokens.take_param(), tokens.take_param(),) {
                                return Ok(MyEnum::Transparent(_0, _1,));
                            }
                            return Err(crate::card_effects::error::Error::UnexpectedToken("MyEnum".into(), s));
                        }
                    })
                }
            }
        };

        assert_eq!(expected_tokens.to_string(), actual_tokens.to_string());
    }
}
