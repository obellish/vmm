#![cfg_attr(docsrs, feature(doc_auto_cfg, doc_cfg))]

use proc_macro::TokenStream as StdTokenStream;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Result, parse_macro_input};

#[proc_macro_derive(BlockProperty)]
pub fn derive_block_property(input: StdTokenStream) -> StdTokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	match create_block_property_impl(input) {
		Ok(ts) => ts.into(),
		Err(err) => err.to_compile_error().into(),
	}
}

#[proc_macro_derive(BlockTransform)]
pub fn derive_block_transform(input: StdTokenStream) -> StdTokenStream {
	let input = parse_macro_input!(input as DeriveInput);

	match create_block_transform_impl(input) {
		Ok(ts) => ts.into(),
		Err(err) => err.to_compile_error().into(),
	}
}

fn create_block_property_impl(input: DeriveInput) -> Result<TokenStream> {
	let fields = match input.data {
		Data::Struct(ds) => ds.fields,
		_ => {
			return Err(Error::new_spanned(
				input,
				"BlockProperty proxy type must be a struct",
			));
		}
	};

	let field_types = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();
	let field_names = fields
		.iter()
		.filter_map(|f| f.ident.as_ref())
		.collect::<Vec<_>>();
	let struct_name = input.ident;

	Ok(quote! {
		impl ::vmm_blocks::BlockProperty for #struct_name {
			fn encode(self, _name: &'static str, props: &mut ::std::collections::HashMap<&'static str, String>) {
				#(
					<#field_types as ::vmm_blocks::BlockProperty>::encode(self.#field_names, stringify!(#field_names), props);
				)*
			}

			fn decode(&mut self, _name: &str, props: &::std::collections::HashMap<&str, &str>) {
				#(
					<#field_types as ::vmm_blocks::BlockProperty>::decode(&mut self.#field_names, stringify!(#field_names), props);
				)*
			}
		}
	})
}

fn create_block_transform_impl(input: DeriveInput) -> Result<TokenStream> {
	let fields = match input.data {
		Data::Struct(ds) => ds.fields,
		_ => {
			return Err(Error::new_spanned(
				input,
				"BlockTransform proxy type must be a struct",
			));
		}
	};

	let field_types = fields.iter().map(|f| &f.ty).collect::<Vec<_>>();
	let field_names = fields
		.iter()
		.filter_map(|f| f.ident.as_ref())
		.collect::<Vec<_>>();
	let struct_name = input.ident;

	Ok(quote! {
		impl ::vmm_blocks::blocks::BlockTransform for #struct_name {
			fn rotate90(&mut self) {
				#(
					<#field_types as ::vmm_blocks::blocks::BlockTransform>::rotate90(&mut self.#field_names);
				)*
			}

			fn flip(&mut self, dir: ::vmm_blocks::blocks::FlipDirection) {
				#(
					<#field_types as ::vmm_blocks::blocks::BlockTransform>::flip(&mut self.#field_names, dir);
				)*
			}
		}
	})
}
