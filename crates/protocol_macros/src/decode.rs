use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Result, parse_quote, parse2, spanned::Spanned};

use super::{add_trait_bounds, decode_split_for_impl};
use crate::pair_variants_with_discriminants;

pub fn derive_decode(item: TokenStream) -> Result<TokenStream> {
	let mut input = parse2::<DeriveInput>(item)?;

	let input_name = input.ident;

	if input.generics.lifetimes().count() > 1 {
		return Err(Error::new(
			input.generics.params.span(),
			"type deriving `Decode` must have no more than one lifetime",
		));
	}

	let lifetime = input
		.generics
		.lifetimes()
		.next()
		.map_or_else(|| parse_quote!('a), |l| l.lifetime.clone());

	match input.data {
		Data::Union(u) => Err(Error::new(
			u.union_token.span(),
			"cannot derive `Decode` on unions",
		)),
		Data::Struct(struct_) => {
			let decode_fields = match struct_.fields {
				Fields::Named(fields) => {
					let init = fields.named.iter().map(|f| {
						let name = f.ident.as_ref().unwrap();

						quote! {
							#name: ::vmm_protocol::__private::Decode::decode(_r)?,
						}
					});

					quote! {
						Self {
							#(#init)*
						}
					}
				}
				Fields::Unnamed(fields) => {
					let init = (0..fields.unnamed.len())
						.map(|_| {
							quote! {
								::vmm_protocol::__private::Decode::decode(_r)?,
							}
						})
						.collect::<TokenStream>();

					quote! {
						Self(#init)
					}
				}
				Fields::Unit => quote!(Self),
			};

			add_trait_bounds(
				&mut input.generics,
				quote!(::vmm_protocol::__private::Decode<#lifetime>),
			);

			let (impl_generics, ty_generics, where_clause) =
				decode_split_for_impl(input.generics, lifetime.clone());

			Ok(quote! {
				impl #impl_generics ::vmm_protocol::__private::Decode<#lifetime> for #input_name #ty_generics
				#where_clause
				{
					fn decode(_r: &mut &#lifetime [u8]) -> ::std::result::Result<Self, ::vmm_protocol::__private::ProtocolError> {
						Ok(#decode_fields)
					}
				}
			})
		}
		Data::Enum(enum_) => {
			let variants = pair_variants_with_discriminants(enum_.variants)?;

			let decode_arms = variants
				.iter()
				.map(|(disc, variant)| {
					let name = &variant.ident;

					match &variant.fields {
						Fields::Named(fields) => {
							let fields = fields
								.named
								.iter()
								.map(|f| {
									let field = f.ident.as_ref().unwrap();

									quote! {
										#field: ::vmm_protocol::__private::Decode::decode(_r)?,
									}
								})
								.collect::<TokenStream>();

							quote! {
								#disc => Ok(Self::#name { #fields }),
							}
						}
						Fields::Unnamed(fields) => {
							let init = (0..fields.unnamed.len())
								.map(|_| {
									quote! {
										::vmm_protocol::__private::Decode::decode(_r)?,
									}
								})
								.collect::<TokenStream>();

							quote! {
								#disc => Ok(Self::#name(#init)),
							}
						}
						Fields::Unit => quote!(#disc => Ok(Self::#name),),
					}
				})
				.collect::<TokenStream>();

			add_trait_bounds(
				&mut input.generics,
				quote!(::vmm_protocol::__private::Decode<#lifetime>),
			);

			let (impl_generics, ty_generics, where_clause) =
				decode_split_for_impl(input.generics, lifetime.clone());

			Ok(quote! {
				impl #impl_generics ::vmm_protocol::__private::Decode<#lifetime> for #input_name #ty_generics
				#where_clause
				{
					fn decode(_r: &mut &#lifetime [u8]) -> ::std::result::Result<Self, ::vmm_protocol::__private::ProtocolError> {
						let disc = ::vmm_protocol::__private::VarInt::decode(_r)?.0;

						match disc {
							#decode_arms
							n => ::std::result::Result::Err(::vmm_protocol::__private::ProtocolError::Other(::std::format!("unexpected enum discriminant {} in `{}`", disc, stringify!(#input_name))))
						}
					}
				}
			})
		}
	}
}
