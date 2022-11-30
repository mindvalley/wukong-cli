mod utils;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn};
use utils::{
    attribute_args_ext::AttributeArgsExt, ident_ext::IdentExt, meta_ext::MetaExt,
    nested_meta_ext::NestedMeta,
};

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

    dbg!(&fn_inputs);

    if let Some(command_event) = command_event_value {
        generated_func = quote! {
            #visibility #asyncness fn #fn_ident(#fn_inputs) #fn_output {
                // SAFETY: the application can't be None since it is checked in the caller
                let current_application = context.application.as_ref().unwrap().clone();
                // SAFETY: the sub can't be None since it is checked in the caller
                let current_sub = context.sub.as_ref().unwrap().clone();

                TelemetryData::new(
                    TelemetryEvent::Command {
                        name: #command_event.to_string(),
                        run_mode: telemetry::CommandRunMode::NonInteractive,
                    },
                    Some(current_application),
                    current_sub,
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
                        if pat_ident.ident.to_string() == "application" {
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

        let telemetry_data;
        if has_application {
            telemetry_data = quote! {
                TelemetryData::new(
                    TelemetryEvent::Api {
                        name: #api_event.to_string(),
                        duration: now.elapsed().as_millis() as u64,
                        response: telemetry::APIResponse::Success,
                    },
                    Some(application.to_string()),
                    "sub".to_string(),
                )
            }
        } else {
            telemetry_data = quote! {
                TelemetryData::new(
                    TelemetryEvent::Api {
                        name: #api_event.to_string(),
                        duration: now.elapsed().as_millis() as u64,
                        response: telemetry::APIResponse::Success,
                    },
                    None,
                    "sub".to_string(),
                )
            }
        }

        generated_func = quote! {
            #visibility #asyncness fn #fn_ident(#fn_inputs) #fn_output {
                let now = std::time::Instant::now();
                let fn_result = #fn_block;

                #telemetry_data
                .record_event()
                .await;

                fn_result
            }
        };
    } else {
        panic!("Expected `command_event` or `api_event` key.");
    }

    generated_func.into()
}
