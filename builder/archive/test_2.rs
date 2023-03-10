use proc_macro::TokenStream;

use quote::quote;
use syn::{self, spanned::Spanned};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let st = syn::parse_macro_input!(input as syn::DeriveInput);
    // We can use below code to check our DeriveInput structure
    // eprintln!("{:#?}", st.data);
    match do_expand(&st) {
        Ok(token_stream) => token_stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

// StructFields is one of the punctuated sequences: https://docs.rs/syn/1.0.109/syn/punctuated/index.html
type StructFields = syn::punctuated::Punctuated<syn::Field, syn::Token!(,)>;

fn get_fields_from_derive_input(d: &syn::DeriveInput) -> syn::Result<&StructFields> {
    if let syn::Data::Struct(syn::DataStruct {
                                 fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
                                 ..
                             }) = d.data {
        return Ok(named);
    }
    Err(syn::Error::new_spanned(d, "Must define on a Struct, not Enum".to_string()))
}

// take get_fields_from_derive_input() output to generate fields and types
fn generate_builder_struct_fields_def(fields: &StructFields) -> syn::Result<proc_macro2::TokenStream> {
    let idents: Vec<_> = fields.iter().map(|f| { &f.ident }).collect();
    let types: Vec<_> = fields.iter().map(|f| { &f.ty }).collect();

    // These will generate belows field and types, will use it inside the new struct.
    // executable: std::option::Option<String>,
    // args: std::option::Option<Vec<String>>,
    // env: std::option::Option<Vec<String>>,
    // current_dir: std::option::Option<String>,
    let token_stream = quote! {
        #(#idents: std::option::Option<#types>),*
    };
    Ok(token_stream)
}

// Notice the return value: Vec<proc_macro2::TokenStream>, later we'll use quote's "*" to expand each element repeatedly.
fn generate_builder_struct_factory_init_clauses(fields: &StructFields) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    // These will generate belows field and types, will use it inside the new struct's builder method.
    // executable: std::option::Option<String>,
    // args: std::option::Option<Vec<String>>,
    // env: std::option::Option<Vec<String>>,
    // current_dir: std::option::Option<String>,
    let init_clauses: Vec<_> = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            #ident: std::option::Option::None
        }
    }).collect();

    Ok(init_clauses)
}

fn do_expand(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    // These take "Command" into "CommandBuilder"
    let struct_name_literal = st.ident.to_string();
    let builder_name_literal = format!("{}Builder", struct_name_literal);
    let builder_name_ident = syn::Ident::new(&builder_name_literal, st.span());

    let struct_ident = &st.ident;

    let fields = get_fields_from_derive_input(st)?;
    let builder_struct_fields_def = generate_builder_struct_fields_def(fields)?;
    let builder_struct_factory_init_clauses = generate_builder_struct_factory_init_clauses(fields)?;

    let ret = quote! {
        pub struct #builder_name_ident {
            #builder_struct_fields_def
        }
        impl #struct_ident {
            pub fn builder() -> #builder_name_ident {
                #builder_name_ident{
                    // expand each field repeatedly since our type is Vec<proc_macro2::TokenStream>
                    #(#builder_struct_factory_init_clauses),*
                }
            }
        }
    };

    return Ok(ret);
}
