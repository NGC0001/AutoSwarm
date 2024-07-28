use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(VectorF32)]
pub fn __derive_impl_vector_f32(item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    let struct_name = &ast.ident;
    let fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(ref fields),
        ..
    }) = ast.data {
        fields
    } else {
        panic!("only support struct")
    };
    let mut idents = vec![];
    for field in &fields.named {
        idents.push(&field.ident);
    }

    quote! {
        impl #struct_name {
            pub fn zero() -> #struct_name {
                #struct_name {
                    #(
                        #idents: 0.0,
                    )*
                }
            }

            #[inline]
            pub fn norm_square(&self) -> f32 {
                #(
                    self.#idents.powi(2)
                )+*
            }

            #[inline]
            pub fn norm(&self) -> f32 {
                self.norm_square().sqrt()
            }
        }

        impl ::std::ops::AddAssign<& #struct_name> for #struct_name {
            fn add_assign(&mut self, other: & #struct_name) {
                #(
                    self.#idents += other.#idents;
                )*
            }
        }
        impl ::std::ops::AddAssign<#struct_name> for #struct_name {
            fn add_assign(&mut self, other: #struct_name) {
                *self += &other;
            }
        }

        impl ::std::ops::Add<& #struct_name> for & #struct_name {
            type Output = #struct_name;
            fn add(self, other: & #struct_name) -> Self::Output {
                #struct_name {
                    #(
                        #idents: self.#idents + other.#idents,
                    )*
                }
            }
        }
        impl ::std::ops::Add<& #struct_name> for #struct_name {
            type Output = #struct_name;
            fn add(self, other: & #struct_name) -> Self::Output {
                &self + other
            }
        }
        impl ::std::ops::Add<#struct_name> for & #struct_name {
            type Output = #struct_name;
            fn add(self, other: #struct_name) -> Self::Output {
                self + &other
            }
        }
        impl ::std::ops::Add<#struct_name> for #struct_name {
            type Output = #struct_name;
            fn add(self, other: #struct_name) -> Self::Output {
                &self + &other
            }
        }

        impl ::std::ops::SubAssign<& #struct_name> for #struct_name {
            fn sub_assign(&mut self, other: & #struct_name) {
                #(
                    self.#idents -= other.#idents;
                )*
            }
        }
        impl ::std::ops::SubAssign<#struct_name> for #struct_name {
            fn sub_assign(&mut self, other: #struct_name) {
                *self -= &other;
            }
        }

        impl ::std::ops::Sub<& #struct_name> for & #struct_name {
            type Output = #struct_name;
            fn sub(self, other: & #struct_name) -> Self::Output {
                #struct_name {
                    #(
                        #idents: self.#idents - other.#idents,
                    )*
                }
            }
        }
        impl ::std::ops::Sub<& #struct_name> for #struct_name {
            type Output = #struct_name;
            fn sub(self, other: & #struct_name) -> Self::Output {
                &self - other
            }
        }
        impl ::std::ops::Sub<#struct_name> for & #struct_name {
            type Output = #struct_name;
            fn sub(self, other: #struct_name) -> Self::Output {
                self - &other
            }
        }
        impl ::std::ops::Sub<#struct_name> for #struct_name {
            type Output = #struct_name;
            fn sub(self, other: #struct_name) -> Self::Output {
                &self - &other
            }
        }

        impl ::std::ops::MulAssign<f32> for #struct_name {
            fn mul_assign(&mut self, multiplier: f32) {
                #(
                    self.#idents *= multiplier;
                )*
            }
        }

        impl ::std::ops::Mul<f32> for & #struct_name {
            type Output = #struct_name;
            fn mul(self, multiplier: f32) -> Self::Output {
                #struct_name {
                    #(
                        #idents: self.#idents * multiplier,
                    )*
                }
            }
        }
        impl ::std::ops::Mul<f32> for #struct_name {
            type Output = #struct_name;
            fn mul(self, multiplier: f32) -> Self::Output {
                &self * multiplier
            }
        }
        impl ::std::ops::Mul<& #struct_name> for f32 {
            type Output = #struct_name;
            fn mul(self, value: & #struct_name) -> Self::Output {
                value * self
            }
        }
        impl ::std::ops::Mul<#struct_name> for f32 {
            type Output = #struct_name;
            fn mul(self, value: #struct_name) -> Self::Output {
                &value * self
            }
        }

        impl ::std::ops::DivAssign<f32> for #struct_name {
            fn div_assign(&mut self, divisor: f32) {
                #(
                    self.#idents /= divisor;
                )*
            }
        }

        impl ::std::ops::Div<f32> for & #struct_name {
            type Output = #struct_name;
            fn div(self, divisor: f32) -> Self::Output {
                #struct_name {
                    #(
                        #idents: self.#idents / divisor,
                    )*
                }
            }
        }
        impl ::std::ops::Div<f32> for #struct_name {
            type Output = #struct_name;
            fn div(self, divisor: f32) -> Self::Output {
                &self / divisor
            }
        }
    }.into()
}