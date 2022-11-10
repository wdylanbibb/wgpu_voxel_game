use proc_macro::TokenStream;

use syn::parse::Parse;
use syn::{Visibility, Ident, Token, Fields, braced, parenthesized, parse_macro_input, ImplItem, Attribute};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Paren};
use quote::{quote, format_ident, ToTokens, TokenStreamExt};

// <vis> enum <enum_name>: <trait> {
//      <TraitEnumFields>, ...
// }
struct TraitEnum {
    attributes: Vec<Attribute>,
    visibility: Visibility,
    _enum_token: Token![enum],
    enum_name: Ident,
    _colon: Token![:],
    enum_trait: Ident,
    _brace_token: Brace,
    fields: Punctuated<TraitEnumFields, Token![,]>,
}

impl Parse for TraitEnum {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(TraitEnum {
            attributes: input.call(Attribute::parse_outer)?,
            visibility: input.parse()?,
            _enum_token: input.parse()?,
            enum_name: input.parse()?,
            _colon: input.parse()?,
            enum_trait: input.parse()?,
            _brace_token: braced!(content in input),
            fields: content.parse_terminated(TraitEnumFields::parse)?,
        })
    }
}

// <name> <info>: {
//      <impls>
// }
struct TraitEnumFields {
    attributes: Vec<Attribute>,
    struct_name: Ident,
    struct_data: ParsableFields,
    impl_block: TraitEnumImpl,
}

impl Parse for TraitEnumFields {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(TraitEnumFields {
            attributes: input.call(Attribute::parse_outer)?,
            struct_name: input.parse()?,
            struct_data: input.parse()?,
            impl_block: input.parse()?,
        })
    }
}

struct ParsableFields {
    fields: Fields,
    semi_token: Option<Token![;]>,
}

impl Parse for ParsableFields {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        let sp = input.span();
        if lookahead.peek(Token![:]) {
            Ok(ParsableFields { fields: Fields::Unit, semi_token: Some(Token![;](sp)) })
        } else if lookahead.peek(Brace) {
            let content;
            Ok(ParsableFields {
                fields: Fields::Named(syn::FieldsNamed {
                    brace_token: braced!(content in input),
                    named: content.parse_terminated(syn::Field::parse_named)?,
                }),
                semi_token: None,
            })
        } else if lookahead.peek(Paren) {
            let content;
            Ok(ParsableFields {
                fields: Fields::Unnamed(syn::FieldsUnnamed {
                    paren_token: parenthesized!(content in input),
                    unnamed: content.parse_terminated(syn::Field::parse_unnamed)?,
                }),
                semi_token: Some(Token![;](sp)),
            })
        } else {
            if lookahead.peek(Token![,]) && !input.peek2(Brace) {
                Ok(ParsableFields { fields: Fields::Unit, semi_token: Some(Token![;](sp)) })
            } else {
                Err(lookahead.error())
            }
        }
    }
}

#[derive(Debug, Clone)]
enum TraitEnumImpl {
    ImplBlock(ImplBlock),
    Empty,
}

impl Parse for TraitEnumImpl {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![:]) {
            Ok(TraitEnumImpl::ImplBlock(input.parse()?))
        } else if lookahead.peek(Token![,]) {
            Ok(TraitEnumImpl::Empty)
        } else {
            Err(lookahead.error())
        }
    }
}

impl ToTokens for TraitEnumImpl {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            TraitEnumImpl::Empty => (),
            TraitEnumImpl::ImplBlock(block) => block.to_tokens(tokens),
        }
    }
}

#[derive(Debug, Clone)]
struct ImplBlock {
    _colon: Token![:],
    _brace_token: Brace,
    items: Vec<ImplItem>,
}

impl Parse for ImplBlock {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(ImplBlock {
            _colon: input.parse()?,
            _brace_token: braced!(content in input),
            items: {
                let mut items = Vec::new();
                while !content.is_empty() {
                    items.push(content.parse()?);
                }
                items
            }
        })
    }
}

impl ToTokens for ImplBlock {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // self._colon.to_tokens(tokens);
        self._brace_token.surround(tokens, |tokens| {
            tokens.append_all(&self.items);
        });
    }
}

pub fn expand_trait_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as TraitEnum);

    // let generate_doc = |f: &str| { syn::parse_str::<syn::Meta>(&format!("doc = \"{}\"", f)).unwrap() };

    let enum_attrs = input.attributes;
    let vis = input.visibility;
    let enum_name = input.enum_name;
    let trait_name = input.enum_trait;
    let struct_attrs = input.fields.iter()
        .map(|f| f.attributes.clone())
        .collect::<Vec<_>>();
    let struct_name = input.fields.iter()
        .map(|f| f.struct_name.clone())
        .collect::<Vec<_>>();
    let struct_data = input.fields.iter()
        .map(|f| f.struct_data.fields.clone())
        .collect::<Vec<_>>();
    let semi = input.fields.iter()
        .map(|f| f.struct_data.semi_token)
        .collect::<Vec<_>>();
    let struct_impl = input.fields.iter()
        .map(|f| f.impl_block.clone())
        .collect::<Vec<_>>();

    let mut struct_construct_name = Vec::new();
    let mut struct_field_idents = Vec::new();
    let mut struct_field_types = Vec::new();
    let mut struct_construct_pattern = Vec::new();
    // Get fields from struct
    for (data, name) in struct_data.iter().zip(struct_name.clone()) {
        struct_construct_name.push(format_ident!("new_{}", name.to_string().to_lowercase()));
        let mut idents = Vec::new();
        let mut types = Vec::new();
        match data {
            Fields::Named(fields_named) => {
                for field in fields_named.named.iter() {
                    idents.push(field.ident.clone().unwrap());
                    types.push(field.ty.clone());
                }

                struct_construct_pattern.push(quote! {
                    {
                        #(#idents),*
                    }
                });
            },
            Fields::Unnamed(fields_unnamed) => {
                for (i, field) in fields_unnamed.unnamed.iter().enumerate() {
                    idents.push(format_ident!("f{}", i));
                    types.push(field.ty.clone());
                }

                struct_construct_pattern.push(quote! {
                    (#(#idents),*)
                });
            },
            Fields::Unit => {
                struct_construct_pattern.push(proc_macro2::TokenStream::new());
            }
        }
        struct_field_idents.push(idents);
        struct_field_types.push(types);
    }

    let struct_impl_tokens = struct_impl.iter().map(|f| match f {
        TraitEnumImpl::ImplBlock(block) => quote! {
            #block
        },
        TraitEnumImpl::Empty => quote! {
            {}
        }
    }).collect::<Vec<_>>();

    let any_trait = format_ident!("{}WithAny", enum_name);

    let enum_attrs_tokens = quote! {
        #(#enum_attrs)*
    };

    let extra_struct_attr = enum_attrs.iter().filter(|x| x.path.is_ident("derive")).collect::<Vec<_>>();

    let extra_struct_attr_tokens = quote! {
        #(#extra_struct_attr)*
    };

    quote! {
        #vis trait #any_trait : #trait_name {
            fn as_any(&self) -> &dyn std::any::Any;
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
        }

        #(
            #extra_struct_attr_tokens
            #(#struct_attrs)*
            #vis struct #struct_name #struct_data #semi
            impl #trait_name for #struct_name
            #struct_impl_tokens
            impl #any_trait for #struct_name {
                fn as_any(&self) -> &dyn std::any::Any { self }
                fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
            }
        )*

        #enum_attrs_tokens
        #vis enum #enum_name {
            #( #struct_name (#struct_name) ),*
        }

        impl std::ops::Deref for #enum_name {
            type Target = dyn #any_trait;

            fn deref(&self) -> &Self::Target {
                match self {
                    #(
                        #enum_name::#struct_name(v) => v,
                    )*
                }
            }
        }

        impl std::ops::DerefMut for #enum_name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                match self {
                    #(
                        #enum_name::#struct_name(v) => v,
                    )*
                }
            }
        }

        impl #enum_name {
            #vis fn get_inner<T>(&self) -> Option<&T> where T: #any_trait + 'static {
                self.deref().as_any().downcast_ref::<T>()
            }

            #vis fn get_inner_mut<T>(&mut self) -> Option<&mut T> where T: #any_trait + 'static {
                self.deref_mut().as_any_mut().downcast_mut::<T>()
            }

            #(
                #vis fn #struct_construct_name(#(#struct_field_idents: #struct_field_types),*) -> #enum_name {
                    #enum_name::#struct_name(#struct_name #struct_construct_pattern)
                }
            )*
        }
    }.into()
}
