#![no_std]
#![recursion_limit = "128"]

extern crate alloc;
extern crate proc_macro;

use alloc::{format, string::String};
use proc_macro::TokenStream;
use proc_macro_roids::namespace_parameters;
use quote::{format_ident, quote};
use syn::{
    parse_quote, Data, DataEnum, DeriveInput, Expr, ExprLit, Fields, FieldsUnnamed, Lit, Meta,
    Path, Variant,
};

// TODO cleanup this file

/// Attributes that should be copied across.

/// Derives a struct for each enum variant.
///
/// Struct fields including their attributes are copied over.
#[cfg(not(tarpaulin_include))]
#[proc_macro_derive(HocgFanSimCardEffect, attributes(hocg_fan_sim))]
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

    let ns: Path = parse_quote!(hocg_fan_sim);

    // serialize effect tokens
    let ser_variants_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let (token, infix, transparent, _default) = attributes(variant, &ns);
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
            Fields::Unit => {
                let token = token.expect("unit enum can only have 'token' attribute");
                quote! { #token.into() }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let mut ns = (0..unnamed.len()).map(|n| {
                    let i = format_ident!("_{}", n);
                    quote! { #i.into() }
                });
                if transparent {
                    quote! { [#(#ns,)*].into() }
                } else {
                    let token = token.as_ref().map(|t| quote! {#t.into(),});
                    let first = ns.next();
                    let infix = infix.as_ref().map(|i| quote! {#i.into(),});
                    quote! { [#token #first, #infix #(#ns,)*].into() }
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

        impl serde::Serialize for #enum_name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                let s = self.clone().serialize_effect();
                String::serialize(&s, serializer)
            }
        }
    };

    // deserialize effect tokens

    let de_variants_tokens2 = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let (token, infix, transparent, _default) = attributes(variant, &ns);
        let variant_fields = &variant.fields;

        if transparent || token.is_none() {
            return quote! {};
        }

        let token = token.unwrap();

        match variant_fields {
            Fields::Unit => quote! {
                //     // - token -
                //     if s == "value" {
                //         return Ok((Value, t));
                //     }
                if s == #token {
                    return Ok((#enum_name::#variant_name, t));
                }
            },
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let ns = (0..unnamed.len()).map(|n| format_ident!("_{}", n));
                let mut take_params = ns.clone().map(|n| {
                    quote! {
                        let (#n, t) = t.take_param()?;
                    }
                });

                if let Some(infix) = infix {
                    //     // - token - infix -
                    //     if s == "let" {
                    //         let (v1, t) = t.take_param()?;
                    //         let (s, t) = t.take_string()?;
                    //         if s == "=" {
                    //             let (v2, t) = t.take_param()?;
                    //             return Ok((Let(v1, v2), t));
                    //         } else {
                    //             return Err(Error::UnexpectedToken(
                    //                 "=".into(),
                    //                 s.clone(),
                    //             ))
                    //         }
                    //     }
                    let first_param = take_params.next();
                    quote! {
                        if s == #token {
                            #first_param
                            let (s, t) = t.take_string()?;
                            if s == #infix {
                                #(#take_params)*
                                return Ok((#enum_name::#variant_name(#(#ns,)*), t));
                            } else {
                                return Err(crate::card_effects::error::Error::UnexpectedToken(#infix.into(), s.clone()));
                            }
                        }
                    }
                } else {
                    //     // - token -
                    //     if s == "value" {
                    //         let (v1, t) = t.take_param()?;
                    //         let (v2, t) = t.take_param()?;
                    //         return Ok((Value(v1, v2), t));
                    //     }
                    quote! {
                        if s == #token {
                            #(#take_params)*
                            return Ok((#enum_name::#variant_name(#(#ns,)*), t));
                        }
                    }
                }
            }
            a => panic!("{:?}", a),
        }
    });
    let has_token = variants.iter().any(|v| attributes(v, &ns).0.is_some());
    let de_variants_tokens2 = if has_token {
        quote! {
            if let Ok((s, t)) = tokens.take_string() {
                #(#de_variants_tokens2)*
            }
        }
    } else {
        quote! {}
    };

    let de_variants_infix2 = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let (token, infix, transparent, _default) = attributes(variant, &ns);
        let variant_fields = &variant.fields;

        if transparent || token.is_some() || infix.is_none() {
            return quote! {};
        }

        let infix = infix.unwrap();

        match variant_fields {
            Fields::Unit => quote! {
                //     // - token -
                //     if s == "value" {
                //         return Ok((Value, t));
                //     }
                let t = tokens;
                if let Ok((s, t)) = t.take_string() {
                    if s == #infix {
                        return Ok((#enum_name::#variant_name, t));
                    }
                }
            },
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let ns = (0..unnamed.len()).map(|n| format_ident!("_{}", n));
                let mut take_params = ns.clone().map(|n| {
                    quote! {
                        let (#n, t) = t.take_param()?;
                    }
                });
                let first_param = take_params.next();

                // // - infix -
                // if let Ok((s, t)) = tokens[1..].take_string() {
                //     if s == "and" {
                //         let t = &tokens[..1];
                //         let (_0, t) = t.take_param()?;
                //         let t = &tokens[2..];
                //         let (_1, t) = t.take_param()?;
                //         return Ok((Infix(_0, _1), t)); // TODO the "t" here is too short, is that true?
                //     }
                // }
                quote! {
                    if s == #infix {
                        let t = &tokens[..1];
                        #first_param
                        let t = &tokens[2..];
                        #(#take_params)*
                        return Ok((#enum_name::#variant_name(#(#ns,)*), t));
                    }
                }
            }
            a => panic!("{:?}", a),
        }
    });
    let has_infix = variants.iter().any(|v| attributes(v, &ns).1.is_some());
    let de_variants_infix2 = if has_infix {
        quote! {
            if let Ok((s, t)) = tokens[1..].take_string() {
                #(#de_variants_infix2)*
            }
        }
    } else {
        quote! {}
    };

    let de_variants_transparent2 = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let (token, infix, transparent, _default) = attributes(variant, &ns);
        let variant_fields = &variant.fields;

        if !transparent || token.is_some() || infix.is_some() {
            return quote! {};
        }

        match variant_fields {
            Fields::Unit => panic!("unit variant cannot be transparent"),
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let ns = (0..unnamed.len()).map(|n| format_ident!("_{}", n));
                let take_params = ns.clone().rev().fold(
                    quote! {
                        return Ok((#enum_name::#variant_name(#(#ns,)*), t));
                    },
                    |acc, n| {
                        quote! {
                            if let Ok((#n, t)) = t.take_param() {
                                #acc
                            }
                        }
                    },
                );

                // // - transparent -
                // if let Ok((v1, t)) = tokens.take_param() {
                //     if let Ok((v2, t)) = t.take_param() {
                //         return Ok((Transparent(v1, v2), t));
                //     }
                // }
                quote! {
                    let t = tokens;
                    #take_params
                }
            }
            a => panic!("{:?}", a),
        }
    });

    let de_variants_default2 = variants.iter().map(|variant| {
        let variant_name = &variant.ident;

        let (_token, _infix, _transparent, default) = attributes(variant, &ns);
        let variant_fields = &variant.fields;

        if !default {
            return quote! {};
        }

        match variant_fields {
            Fields::Unit => quote! {
                fn default_effect() -> Option<Self> {
                    Some(#enum_name::#variant_name)
                }
            },
            a => panic!("only unit variant can be default: {:?}", a),
        }
    });
    let has_default = variants.iter().any(|v| attributes(v, &ns).3);
    let de_variants_default2 = if has_default {
        quote! {
            #(#de_variants_default2)*
        }
    } else {
        quote! {
            fn default_effect() -> Option<Self> {
                None
            }
        }
    };

    // // - infix -
    // if let Ok((s, t)) = tokens[1..].take_string() {
    //     if s == "and" {
    //         let t = &tokens[..1];
    //         let (_0, t) = t.take_param()?;
    //         let t = &tokens[2..];
    //         let (_1, t) = t.take_param()?;
    //         return Ok((Infix(_0, _1), t)); // TODO the "t" here is too short, is that true?
    //     }
    // }
    // // - token -
    // if let Ok((s, t)) = tokens.take_string() {
    //     if s == "value" {
    //         let (v1, t) = t.take_param()?;
    //         let (v2, t) = t.take_param()?;
    //         return Ok((Value(v1, v2), t));
    //     }
    //     if s == "value2" {
    //         let (v1, t) = t.take_param()?;
    //         let (v2, t) = t.take_param()?;
    //         return Ok((Value2(v1, v2), t));
    //     }
    //     // - token - infix -
    //     if s == "let" {
    //         let (v1, t) = t.take_param()?;
    //         let (s, t) = t.take_string()?;
    //         if s == "=" {
    //             let (v2, t) = t.take_param()?;
    //             return Ok((Let(v1, v2), t));
    //         } else {
    //             return Err(Error::UnexpectedToken(
    //                 "=".into(),
    //                 s.clone(),
    //             ))
    //         }
    //     }
    // }
    // // - transparent -
    // if let Ok((v1, t)) = tokens.take_param() {
    //     if let Ok((v2, t)) = t.take_param() {
    //         return Ok((Transparent(v1, v2), t));
    //     }
    // }
    // if let Ok((v1, t)) = tokens.take_param() {
    //     if let Ok((v2, t)) = t.take_param() {
    //         return Ok((Transparent2(v1, v2), t));
    //     }
    // }
    // Err(Error::UnexpectedToken(
    //     "LetValue".into(),
    //     tokens.take_string()?.0.clone(),
    // ))
    let str_enum_name = format!("{enum_name}");
    let impl_de_for_enum = quote! {
        impl crate::card_effects::parse::ParseTokens for #enum_name {

            #de_variants_default2

            fn parse_tokens(tokens: &[crate::card_effects::parse::Tokens]) -> std::result::Result<(Self, &[crate::card_effects::parse::Tokens]), crate::card_effects::error::Error> {
                if tokens.is_empty() {
                    return Err(crate::card_effects::error::Error::ExpectedToken);
                }

                // println!("{:?} - {:?}", #str_enum_name, tokens);

                #de_variants_infix2
                #de_variants_tokens2
                #(#de_variants_transparent2)*

                return Err(crate::card_effects::error::Error::UnexpectedToken(#str_enum_name.into(), tokens.take_string()?.0.clone()));
            }
        }

        impl<'de> serde::Deserialize<'de> for #enum_name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let s = String::deserialize(deserializer)?;
                crate::card_effects::parse::ParseTokens::from_str(&s).map_err(serde::de::Error::custom)
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

fn attributes(variant: &Variant, ns: &Path) -> (Option<String>, Option<String>, bool, bool) {
    let evt_meta_lists = namespace_parameters(&variant.attrs, ns);
    let mut token = None;
    let mut infix = None;
    let mut transparent = false;
    let mut default = false;
    for meta in evt_meta_lists {
        match meta {
            Meta::NameValue(name_value) => {
                if let (
                    true,
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }),
                ) = (name_value.path.is_ident("token"), &name_value.value)
                {
                    token = Some(lit_str.value());
                } else if let (
                    true,
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }),
                ) = (name_value.path.is_ident("infix"), &name_value.value)
                {
                    infix = Some(lit_str.value());
                } else {
                    panic!("Expected `hocg_fan_sim` attribute argument in the form: `#[hocg_fan_sim(token = \"some_token\")]`");
                }
            }
            Meta::Path(path) => {
                if path.is_ident("transparent") {
                    transparent = true;
                } else if path.is_ident("default") {
                    default = true;
                } else {
                    panic!("Expected `hocg_fan_sim` attribute argument in the form: `#[hocg_fan_sim(transparent)]`");
                }
            }
            a => panic!("{:?}", a),
        }
    }
    assert!(
        token.is_some() || infix.is_some() || transparent,
        "Expected to have at least one of (token, infix, transparent)"
    );
    (token, infix, transparent, default)
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
                #[evt(derive(Clone, Copy, Debug, PartialEq, Eq))]
                #[hocg_fan_sim(default, token = "unit", infix = "=")]
                Unit,
                /// Tuple variant.
                #[evt(derive(Debug))]
                #[hocg_fan_sim(token = "tuple")]
                Tuple(u32, u64),
                // /// Tuple variant.
                #[evt(derive(Debug))]
                #[hocg_fan_sim(infix = "+")]
                TupleInfix(u32, u64),
                /// Tuple variant.
                #[evt(derive(Debug))]
                #[hocg_fan_sim(token = "let", infix = "=")]
                TuplePrefixInfix(u32, u64),
                /// Tuple variant.
                #[evt(derive(Debug))]
                #[hocg_fan_sim(transparent)]
                Transparent(u32, u32),
            }
        };

        // // - infix -
        // if let Ok((s, t)) = tokens[1..].take_string() {
        //     if s == "and" {
        //         let t = &tokens[..1];
        //         let (_0, t) = t.take_param()?;
        //         let t = &tokens[2..];
        //         let (_1, t) = t.take_param()?;
        //         return Ok((Infix(_0, _1), t)); // TODO the "t" here is too short, is that true?
        //     }
        // }
        // // - token -
        // if let Ok((s, t)) = tokens.take_string() {
        //     if s == "value" {
        //         let (v1, t) = t.take_param()?;
        //         let (v2, t) = t.take_param()?;
        //         return Ok((Value(v1, v2), t));
        //     }
        //     if s == "value2" {
        //         let (v1, t) = t.take_param()?;
        //         let (v2, t) = t.take_param()?;
        //         return Ok((Value2(v1, v2), t));
        //     }
        //     // - token - infix -
        //     if s == "let" {
        //         let (v1, t) = t.take_param()?;
        //         let (s, t) = t.take_string()?;
        //         if s == "=" {
        //             let (v2, t) = t.take_param()?;
        //             return Ok((Let(v1, v2), t));
        //         } else {
        //             return Err(Error::UnexpectedToken(
        //                 "=".into(),
        //                 s.clone(),
        //             ))
        //         }
        //     }
        // }
        // // - transparent -
        // if let Ok((v1, t)) = tokens.take_param() {
        //     if let Ok((v2, t)) = t.take_param() {
        //         return Ok((Transparent(v1, v2), t));
        //     }
        // }
        // if let Ok((v1, t)) = tokens.take_param() {
        //     if let Ok((v2, t)) = t.take_param() {
        //         return Ok((Transparent2(v1, v2), t));
        //     }
        // }
        // Err(Error::UnexpectedToken(
        //     "LetValue".into(),
        //     tokens.take_string()?.0.clone(),
        // ))

        let actual_tokens = ser_de_token_for_enum_impl(ast);
        let expected_tokens = quote! {
            impl core::convert::From<MyEnum> for crate::card_effects::parse::Tokens {
                fn from(value: MyEnum) -> Self {
                    match value {
                        MyEnum::Unit => "unit".into(),
                        MyEnum::Tuple(_0, _1,) => ["tuple".into(), _0.into(), _1.into(),].into(),
                        MyEnum::TupleInfix(_0, _1,) => [_0.into(), "+".into(), _1.into(),].into(),
                        MyEnum::TuplePrefixInfix(_0, _1,) =>
                            ["let".into(), _0.into(), "=".into(), _1.into(),].into(),
                        MyEnum::Transparent(_0, _1,) => [_0.into(), _1.into(),].into(),
                    }
                }
            }
            impl serde::Serialize for MyEnum {
                fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
                where
                    S: serde::Serializer,
                {
                    let s = self.clone().serialize_effect();
                    String::serialize(&s, serializer)
                }
            }
            impl crate::card_effects::parse::ParseTokens for MyEnum {
                fn default_effect() -> Option<Self> {
                    Some(MyEnum::Unit)
                }
                fn parse_tokens(
                    tokens: &[crate::card_effects::parse::Tokens]
                ) -> std::result::Result<
                    (Self, &[crate::card_effects::parse::Tokens]),
                    crate::card_effects::error::Error
                > {
                    if tokens.is_empty() {
                        return Err(crate::card_effects::error::Error::ExpectedToken);
                    }
                    if let Ok((s, t)) = tokens[1..].take_string() {
                        if s == "+" {
                            let t = &tokens[..1];
                            let (_0, t) = t.take_param()?;
                            let t = &tokens[2..];
                            let (_1, t) = t.take_param()?;
                            return Ok((MyEnum::TupleInfix(_0, _1,), t));
                        }
                    }
                    if let Ok((s, t)) = tokens.take_string() {
                        if s == "unit" {
                            return Ok((MyEnum::Unit, t));
                        }
                        if s == "tuple" {
                            let (_0, t) = t.take_param()?;
                            let (_1, t) = t.take_param()?;
                            return Ok((MyEnum::Tuple(_0, _1,), t));
                        }
                        if s == "let" {
                            let (_0, t) = t.take_param()?;
                            let (s, t) = t.take_string()?;
                            if s == "=" {
                                let (_1, t) = t.take_param()?;
                                return Ok((MyEnum::TuplePrefixInfix(_0, _1,), t));
                            } else {
                                return Err(crate::card_effects::error::Error::UnexpectedToken(
                                    "=".into(),
                                    s.clone()
                                ));
                            }
                        }
                    }
                    let t = tokens;
                    if let Ok((_0, t)) = t.take_param() {
                        if let Ok((_1, t)) = t.take_param() {
                            return Ok((MyEnum::Transparent(_0, _1,), t));
                        }
                    }
                    return Err(crate::card_effects::error::Error::UnexpectedToken(
                        "MyEnum".into(),
                        tokens.take_string()?.0.clone()
                    ));
                }
            }
            impl<'de> serde::Deserialize<'de> for MyEnum {
                fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    let s = String::deserialize(deserializer)?;
                    crate::card_effects::parse::ParseTokens::from_str(&s).map_err(serde::de::Error::custom)
                }
            }
        };

        assert_eq!(expected_tokens.to_string(), actual_tokens.to_string());
    }
}
