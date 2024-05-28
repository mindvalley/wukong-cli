// https://github.com/nazmulidris/rust_scratch/blob/main/macros/my_proc_macros_lib/src/utils/attribute_args_ext.rs
use crate::utils::{meta_ext::MetaExt, nested_meta_ext::NestedMeta};

pub trait AttributeArgsExt {
    #[allow(dead_code)]
    fn get_key_value_pair(&self) -> (String, String);
    fn get_value(&self, key: &str) -> Option<String>;
}

/// The args take a key value pair like `#[attrib_macro_logger(key = "value")]`, which
/// evaluates to:
/// ```no_run
/// &args = [
///     Meta(
///         NameValue(
///             MetaNameValue {
///                 path: Path {
///                     leading_colon: None,
///                     segments: [
///                         PathSegment {
///                             ident: Ident {
///                                 ident: "key",
///                                 span: #0 bytes(510..513),
///                             },
///                             arguments: None,
///                         },
///                     ],
///                 },
///                 eq_token: Eq,
///                 lit: Str(
///                     LitStr {
///                         token: "value",
///                     },
///                 ),
///             },
///         ),
///     ),
/// ]
/// ```
impl AttributeArgsExt for syn::AttributeArgs {
    fn get_key_value_pair(&self) -> (String, String) {
        for nested_meta in self.iter() {
            if nested_meta.is_meta() {
                let meta = nested_meta.get_meta();
                if meta.is_meta_name_value() {
                    let key = meta.get_meta_name_value_ident().to_string();
                    let value = meta.get_meta_name_value_str();
                    return (key, value);
                }
            }
        }
        panic!("Expected a key value pair");
    }

    fn get_value(&self, key: &str) -> Option<String> {
        for nested_meta in self.iter() {
            if nested_meta.is_meta() {
                let meta = nested_meta.get_meta();
                if meta.is_meta_name_value() {
                    let meta_key = meta.get_meta_name_value_ident().to_string();
                    if meta_key == key {
                        let value = meta.get_meta_name_value_str();
                        return Some(value);
                    }
                }
            }
        }
        None
        // panic!("Expected a key `{}`.", key);
    }
}
