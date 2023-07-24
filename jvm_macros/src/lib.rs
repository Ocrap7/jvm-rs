use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Field, Meta, Type, Fields};

enum AttrData {
    Many(Type),
}

fn get_attribute(attrs: &Vec<Attribute>, name: &str) -> Option<AttrData> {
    let attribute = attrs.iter().find(|attr| match &attr.meta {
        Meta::Path(path) => path.get_ident().unwrap().to_string() == name,
        Meta::List(list) => list.path.get_ident().unwrap().to_string() == name,
        _ => false,
    })?;

    match name.to_lowercase().as_str() {
        "many" => {
            let Meta::List(list) = &attribute.meta else {
                return None;
            };

            Some(AttrData::Many(syn::parse2(list.tokens.clone()).unwrap()))
        }
        _ => None,
    }
}

#[proc_macro_derive(StreamReader, attributes(many))]
pub fn derive_stream_reader(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(tokens as syn::DeriveInput);

    let ty_ident = &input.ident;

    let tokens = match input.data {
        syn::Data::Struct(data) => {
            let mut out_tokens = TokenStream::new();

            for field in &data.fields {
                let Field {
                    ident, attrs, ..
                } = &field;

                match get_attribute(attrs, "many") {
                    Some(AttrData::Many(ty)) => {
                        if let Some(ident) = ident {
                            out_tokens.extend(quote! {
                                #ident: {
                                    let len = stream.read::<#ty>(ctx);
                                    stream.read_many(len as usize, ctx)
                                },
                            });
                        } else {
                            out_tokens.extend(quote! {
                                {
                                    let len = stream.read::<#ty>();
                                    stream.read_many(len as usize, ctx)
                                },
                            });
                        }
                    },
                    _ => {
                        if let Some(ident) = ident {
                            out_tokens.extend(quote! {
                                #ident: stream.read(ctx),
                            });
                        } else {
                            out_tokens.extend(quote! {stream.read(ctx), });
                        }
                    }
                }
            }

            match &data.fields {
                Fields::Named(_) => {
                    quote! {
                        Self {
                            #out_tokens
                        }
                    }
                }
                Fields::Unnamed(_) => {
                    quote! {
                        Self(#out_tokens)
                    }
                }
                Fields::Unit=> {
                    quote!{Self}
                }
            }
        }
        _ => TokenStream::new(),
    };

    let tokens = quote! {
        impl crate::byte_stream::StreamRead for #ty_ident {
            #[allow(dead_code)]
            fn read<'a>(stream: &mut crate::byte_stream::ByteStream<'a>, ctx: &crate::byte_stream::ReaderContext) -> Self {
                #tokens
            }
        }
    };

    tokens.into()
}
