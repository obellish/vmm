use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, LitInt, Result, parse2, spanned::Spanned};

use super::{add_trait_bounds, pair_variants_with_discriminants};

pub fn derive_encode(item: TokenStream) -> Result<TokenStream> {
	let mut input = parse2::<DeriveInput>(item)?;

	let input_name = input.ident;

	add_trait_bounds(
		&mut input.generics,
		quote!(::vmm_protocol::__private::Encode),
	);

	let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

	match input.data {
		Data::Union(u) => Err(Error::new(
			u.union_token.span(),
			"cannot derive `Encode` on unions",
		)),
		Data::Struct(struct_) => {
			let encode_fields = match &struct_.fields {
                Fields::Named(fields) => fields.named.iter().map(|f| {
                    let name = &f.ident.as_ref().unwrap();
                    let ctx = format!("failed to encode field `{name}` in `{input_name}`");

                    quote! {
                        self.#name.encode(&mut _w).map_err(|_| ::vmm_protocol::__private::ProtocolError::Other(#ctx.to_owned()))?;
                    }
                })
                .collect(),
                Fields::Unnamed(fields) => (0..fields.unnamed.len()).map(|i| {
                    let lit = LitInt::new(&i.to_string(), Span::call_site());
                    let ctx = format!("failed to encode field `{lit}` in `{input_name}`");

                    quote! {
                        self.#lit.encode(&mut _w).map_err(|_| ::vmm_protocol::__private::ProtocolError::Other(#ctx.to_owned()))?;
                    }
                })
                .collect(),
                Fields::Unit => TokenStream::new()
            };

			Ok(quote! {
				#[allow(unused_imports)]
				impl #impl_generics ::vmm_protocol::__private::Encode for #input_name #ty_generics
				#where_clause
				{
					fn encode(&self, mut _w: impl ::std::io::Write) -> ::std::result::Result<(), ::vmm_protocol::__private::ProtocolError> {
						use ::vmm_protocol::__private::Encode;

						#encode_fields;

						Ok(())
					}
				}
			})
		}
		Data::Enum(enum_) => {
			let variants = pair_variants_with_discriminants(enum_.variants)?;

			let encode_arms = variants
				.iter()
				.map(|(disc, variant)| {
					let variant_name = &variant.ident;

					match &variant.fields {
						Fields::Named(fields) => {
							let field_names = fields
								.named
								.iter()
								.map(|f| f.ident.as_ref().unwrap())
								.collect::<Vec<_>>();

							let encode_fields = field_names
								.iter()
								.map(|name| {
									quote! {
										#name.encode(&mut _v)?;
									}
								})
								.collect::<TokenStream>();

							quote! {
								Self::#variant_name {#(#field_names,)* } => {
									::vmm_protocol::__private::VarInt(#disc).encode(&mut _w)?;

									#encode_fields;
									Ok(())
								}
							}
						}
						Fields::Unnamed(fields) => {
							let field_names = (0..fields.unnamed.len())
								.map(|i| Ident::new(&format!("_{i}"), Span::call_site()))
								.collect::<Vec<_>>();

							let encode_fields = field_names
								.iter()
								.map(|name| {
									quote! {
										#name.encode(&mut _w)?;
									}
								})
								.collect::<TokenStream>();

							quote! {
								Self::#variant_name(#(#field_names,)*) => {
									::vmm_protocol::__private::VarInt(#disc).encode(&mut _w)?;

									#encode_fields;
									Ok(())
								}
							}
						}
						Fields::Unit => quote! {
							Self::#variant_name => Ok(::vmm_protocol::__private::VarInt(#disc).encode(&mut _w)?),
						},
					}
				})
				.collect::<TokenStream>();

			Ok(quote! {
				impl #impl_generics ::vmm_protocol::__private::Encode for #input_name #ty_generics
				#where_clause
				{
					fn encode(&self, mut _w: impl ::std::io::Write) -> ::std::result::Result<(), ::vmm_protocol::__private::ProtocolError> {
						match self {
							#encode_arms
							_ => unreachable!(),
						}
					}
				}
			})
		}
	}
}
