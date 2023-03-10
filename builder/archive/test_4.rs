use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let st = syn::parse_macro_input!(input as syn::DeriveInput);
    match do_expand(&st) {
        Ok(token_stream) => token_stream.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

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

fn generate_builder_struct_fields_def(fields: &StructFields) -> syn::Result<proc_macro2::TokenStream> {
    let idents: Vec<_> = fields.iter().map(|f| { &f.ident }).collect();
    let types: Vec<_> = fields.iter().map(|f| { &f.ty }).collect();

    let token_stream = quote! {
        #(#idents: std::option::Option<#types>),*
    };
    Ok(token_stream)
}

fn generate_builder_struct_factory_init_clauses(fields: &StructFields) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let init_clauses: Vec<_> = fields.iter().map(|f| {
        let ident = &f.ident;
        quote! {
            #ident: std::option::Option::None
        }
    }).collect();

    Ok(init_clauses)
}

fn generate_setter_functions(fields: &StructFields) -> syn::Result<proc_macro2::TokenStream> {
    let idents: Vec<_> = fields.iter().map(|f| { &f.ident }).collect();
    let types: Vec<_> = fields.iter().map(|f| { &f.ty }).collect();

    let mut final_tokenstream = proc_macro2::TokenStream::new();

    for (ident, type_) in idents.iter().zip(types.iter()) {
        let tokenstream_piece = quote! {
            fn #ident(&mut self, #ident: #type_) -> &mut Self {
                self.#ident = std::option::Option::Some(#ident);
                self
            }
        };
        final_tokenstream.extend(tokenstream_piece);
    }

    Ok(final_tokenstream)
}

fn generate_build_function(fields: &StructFields, origin_struct_ident: &syn::Ident) -> syn::Result<proc_macro2::TokenStream> {
    let idents: Vec<_> = fields.iter().map(|f| { &f.ident }).collect();

    let mut checker_code_pieces = Vec::new();
    for idx in 0..idents.len() {
        let ident = idents[idx];
        checker_code_pieces.push(quote! {
            if self.#ident.is_none() {
                let err = format!("{} field missing", stringify!(#ident));
                return std::result::Result::Err(err.into())
            }
        });
    }

    let mut fill_result_clauses = Vec::new();
    for idx in 0..idents.len() {
        let ident = idents[idx];
        fill_result_clauses.push(quote! {
            #ident: self.#ident.clone().unwrap()
        });
    }


    let token_stream = quote! {
        pub fn build(&mut self) -> std::result::Result<#origin_struct_ident, std::boxed::Box<dyn std::error::Error>> {
            #(#checker_code_pieces)*
                               //  ^--Note that because we want to repeat a set of IF judgment code blocks,
                               //     they do not need to be separated by comma, so the repeated mode here is `*`, not `,*` that the previous duplicate structure field is used.
            // build a Command struct
            let ret = #origin_struct_ident{
                #(#fill_result_clauses),*
            };
            std::result::Result::Ok(ret)
        }
    };
    Ok(token_stream)
}

fn do_expand(st: &syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name_literal = st.ident.to_string();
    let builder_name_literal = format!("{}Builder", struct_name_literal);
    let builder_name_ident = syn::Ident::new(&builder_name_literal, st.span());

    let struct_ident = &st.ident;

    let fields = get_fields_from_derive_input(st)?;
    let builder_struct_fields_def = generate_builder_struct_fields_def(fields)?;
    let builder_struct_factory_init_clauses = generate_builder_struct_factory_init_clauses(fields)?;
    let setter_functions = generate_setter_functions(fields)?;
    // for test 4
    let generated_builder_functions = generate_build_function(fields, struct_ident)?;

    let ret = quote! {
        pub struct #builder_name_ident {
            #builder_struct_fields_def
        }
        impl #struct_ident {
            pub fn builder() -> #builder_name_ident {
                #builder_name_ident{
                    #(#builder_struct_factory_init_clauses),*
                }
            }
        }
        impl #builder_name_ident {
            #setter_functions
            // for test 4
            #generated_builder_functions
        }
    };

    return Ok(ret);
}