use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use syn::{
    AttributeArgs, Data, DataEnum, DataStruct, DeriveInput, Fields, Ident, ItemFn, Meta,
    NestedMeta, Type, parse_macro_input,
};

#[proc_macro_derive(Handler, attributes(handler_inject))]
#[proc_macro_error]
pub fn derive_handler(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let context_injector = match input
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident("handler_inject"))
        .and_then(|a| a.parse_args::<Ident>().ok())
    {
        Some(ident) => {
            cfg_if::cfg_if! {
                if #[cfg(feature = "async")] {
                    quote! { self.#ident(ctx).await?; }
                } else {
                    quote! { self.#ident(ctx)?; }
                }
            }
        }
        None => quote! {},
    };

    let expanded = match input.data {
        Data::Struct(DataStruct { fields, .. }) => match fields {
            Fields::Named(ref fields_name) => {
                let subcommand_field: Option<syn::Ident> =
                    fields_name.named.iter().find_map(|field| {
                        for attr in field.attrs.iter() {
                            if attr.path.is_ident("clap") {
                                let ident: syn::Ident = attr.parse_args().ok()?;
                                if ident == "subcommand" {
                                    return Some(field.ident.clone().unwrap());
                                }
                            }
                        }
                        None
                    });

                match subcommand_field {
                    Some(subcommand_field) => {
                        #[cfg(not(feature = "async"))]
                        quote! {
                            impl clap_handler::Handler for #name {
                                fn handle_command(&mut self, ctx: &mut clap_handler::Context) -> anyhow::Result<()> {
                                    #context_injector
                                    Ok(())
                                }

                                fn handle_subcommand(&mut self, ctx: clap_handler::Context) -> anyhow::Result<()> {
                                    clap_handler::Handler::execute(&mut self.#subcommand_field, ctx)
                                }
                            }
                        }

                        #[cfg(feature = "async")]
                        quote! {
                            #[clap_handler::async_trait]
                            impl clap_handler::Handler for #name {
                                async fn handle_command(&mut self, ctx: &mut clap_handler::Context) -> anyhow::Result<()> {
                                    #context_injector
                                    Ok(())
                                }

                                async fn handle_subcommand(&mut self, ctx: clap_handler::Context) -> anyhow::Result<()> {
                                    clap_handler::Handler::execute(&mut self.#subcommand_field, ctx).await
                                }
                            }
                        }
                    }
                    None => panic!("Struct without #[clap(subcommand)] is not supported!"),
                }
            }
            _ => panic!("Unnamed fields or None struct is not supported"),
        },
        Data::Enum(DataEnum { variants, .. }) => {
            let subcommands: Vec<_> = variants
                .iter()
                .map(|v| {
                    let ident = &v.ident;
                    quote! { #name::#ident }
                })
                .collect();
            #[cfg(not(feature = "async"))]
            quote! {
                impl clap_handler::Handler for #name {
                    fn execute(&mut self, mut ctx: clap_handler::Context) -> anyhow::Result<()> {
                        match self {
                            #(#subcommands(s) => clap_handler::Handler::execute(s, ctx),)*
                        }
                    }
                }
            }
            #[cfg(feature = "async")]
            quote! {
                #[clap_handler::async_trait]
                impl clap_handler::Handler for #name {
                    async fn execute(&mut self, mut ctx: clap_handler::Context) -> anyhow::Result<()> {
                        match self {
                            #(#subcommands(s) => clap_handler::Handler::execute(s, ctx).await,)*
                        }
                    }
                }
            }
        }
        _ => panic!("Union type is not supported"),
    };
    expanded.into()
}

#[proc_macro_attribute]
#[proc_macro_error]
pub fn handler(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(args as AttributeArgs);
    let attr = match attr.get(0).as_ref().unwrap() {
        NestedMeta::Meta(Meta::Path(attr_ident)) => attr_ident.get_ident().unwrap(),
        _ => unreachable!("it not gonna happen."),
    };

    let func = parse_macro_input!(input as ItemFn);
    let func_block = &func.block;
    let func_sig = func.sig;
    let func_name = &func_sig.ident;
    let func_generics = &func_sig.generics;
    let func_inputs = &func_sig.inputs;
    let func_output = &func_sig.output;
    let types: Vec<_> = func_inputs
        .iter()
        .map(|i| {
            match i {
                syn::FnArg::Typed(ty) => {
                    let ty: &Type = &ty.ty;
                    match ty {
                        Type::Reference(r) => {
                            if r.mutability.is_some() {
                                quote! { ctx.get_mut().unwrap() }
                            } else {
                                quote! { ctx.get().unwrap() }
                            }
                        }
                        _ => {
                            // owned type
                            // TODO: do not unwrap when ty is Option<T>
                            // TODO: do not deref when ty is Box<T>
                            quote! { *ctx.take().unwrap() }
                        }
                    }
                }
                _ => unreachable!("syntax error"),
            }
        })
        .collect();

    cfg_if::cfg_if! {
        if #[cfg(feature = "async")] {
            let expanded = quote! {
                #[clap_handler::async_trait]
                impl clap_handler::Handler for #attr {
                    async fn handle_command(&mut self, ctx: &mut clap_handler::Context) -> anyhow::Result<()> {
                        async fn #func_name #func_generics(#func_inputs)#func_output {
                            #func_block
                        }
                        let result = #func_name(#(#types,)*);
                        Ok(result.await?)
                    }
                }
            };
        } else {
            let expanded = quote! {
                impl clap_handler::Handler for #attr {
                    fn handle_command(&mut self, ctx: &mut clap_handler::Context) -> anyhow::Result<()> {
                        fn #func_name #func_generics(#func_inputs)#func_output {
                            #func_block
                        }
                        let result = #func_name(#(#types,)*);
                        Ok(result?)
                    }
                }
            };
        }
    }

    expanded.into()
}
