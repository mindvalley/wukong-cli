#[cfg(not(doctest))]
mod utils;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn};
#[cfg(not(doctest))]
use utils::attribute_args_ext::AttributeArgsExt;

#[proc_macro_attribute]
pub fn wukong_telemetry(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let item = parse_macro_input!(item as ItemFn);

    let command_event_value = args.get_value("command_event");
    let api_event_value = args.get_value("api_event");

    let fn_ident = item.sig.ident;
    let fn_inputs = item.sig.inputs;
    let fn_output = item.sig.output;
    let fn_block = item.block;
    let asyncness = item.sig.asyncness;
    let visibility = item.vis;

    let generated_func;

    if let Some(command_event) = command_event_value {
        generated_func = quote! {
            #[allow(clippy::too_many_arguments)]
            #visibility #asyncness fn #fn_ident(#fn_inputs) #fn_output {
                use clap::{crate_version};

                let current_application = context.current_application.clone();
                let current_sub = match context.sub {
                    Some(sub) => sub,
                    None => "unknown".to_string(),
                };

                TelemetryData::new(
                    TelemetryEvent::Command {
                        name: #command_event.to_string(),
                        run_mode: CommandRunMode::NonInteractive,
                    },
                    Some(current_application),
                    current_sub,
                    crate_version!().to_string()
                )
                .record_event()
                .await;

                #fn_block
            }
        };
    } else if let Some(api_event) = api_event_value {
        let has_application = fn_inputs
            .iter()
            .find_map(|input| match input {
                syn::FnArg::Typed(typed) => match &*typed.pat {
                    syn::Pat::Ident(pat_ident) => {
                        if pat_ident.ident == "application" {
                            Some(())
                        } else {
                            None
                        }
                    }
                    _ => None,
                },
                _ => None,
            })
            .is_some();

        let current_application = if has_application {
            quote! { Some(application.to_string()) }
        } else {
            quote! { None }
        };

        generated_func = quote! {
            #visibility #asyncness fn #fn_ident(#fn_inputs) #fn_output {
                use clap::{crate_version};

                let current_sub = self.sub.clone();

                let now = std::time::Instant::now();

                let fn_result = #fn_block;

                let telemetry_data = match fn_result {
                    Ok(_) => {
                        TelemetryData::new(
                            TelemetryEvent::Api {
                                name: #api_event.to_string(),
                                duration: now.elapsed().as_millis() as u64,
                                response: APIResponse::Success,
                            },
                            #current_application,
                            current_sub,
                            crate_version!().to_string()
                        )
                    },
                    Err(_) => {
                        TelemetryData::new(
                            TelemetryEvent::Api {
                                name: #api_event.to_string(),
                                duration: now.elapsed().as_millis() as u64,
                                response: APIResponse::Error,
                            },
                            #current_application,
                            current_sub,
                            crate_version!().to_string()
                        )
                    }
                };

                telemetry_data
                .record_event()
                .await;

                fn_result.map_err(|err| err.into())
            }
        };
    } else {
        panic!("Expected `command_event` or `api_event` key.");
    }

    generated_func.into()
}
