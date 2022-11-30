// https://github.com/nazmulidris/rust_scratch/blob/main/macros/my_proc_macros_lib/src/utils/nested_meta_ext.rs

pub trait NestedMeta {
  fn is_meta(&self) -> bool;
  fn get_meta(&self) -> &syn::Meta;
}

/// Can be either a 👉 [syn::NestedMeta::Meta] or a [syn::NestedMeta::Lit].
impl NestedMeta for syn::NestedMeta {
  fn is_meta(&self) -> bool {
    match self {
      syn::NestedMeta::Meta(_) => true,
      syn::NestedMeta::Lit(_) => false,
    }
  }

  fn get_meta(&self) -> &syn::Meta {
    match self {
      syn::NestedMeta::Meta(meta) => meta,
      syn::NestedMeta::Lit(_) => panic!("Lit found"),
    }
  }
}
