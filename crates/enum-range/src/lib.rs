extern crate proc_macro;
use darling::{FromMeta, FromVariant};
use proc_macro::TokenStream;
use proc_macro2::{Ident, Literal, Span};
use quote::{format_ident, quote, ToTokens};
use regex::Regex;
use std::collections::VecDeque;
use std::str::FromStr;
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, Token, Variant};

/// Represents the instructions defining a range of values in an enum
///
/// ```rust
/// use enum_range::enum_range;
///
/// #[enum_range]
/// #[repr(u8)]
/// enum RangedEnum {
///     NormalVariant = 1,
///     #[range(
///         format = "WellKnown{index}",
///         start = 206,
///         end = 210,
///         range_check = "is_well_known"
///     )]
///     RangeVariant,
///     OtherNormalVariant = 211,
/// }
/// ```
///
/// Parameters:
/// - `start`: the first variant discriminator value in the range (start is included)
/// - `end`: the last variant discriminator value in the range (end is included)
/// - `format` (optional): the format used for naming the different variants.
/// `{index}` is replaced by the index of the variant in the defined range (here 0-5)
/// `{value}` is replaced by the value of the variant in the defined range (here 206-210)
/// The default value is `"VariantName{index}"`
/// - `range_check` (optional): the name of the method used to check if an enum variant is in the defined range (here `RangedEnum::is_well_known`).
/// This only works if the enum has a numerical repr attribute. If either `range_check` or `repr` are not specified the method is not generated.
///
#[derive(Debug, Default, FromVariant)]
#[darling(default, attributes(range))]
struct Range {
    format: Option<String>,
    start: usize,
    end: usize,
    range_check: Option<String>,
}

/// Main derive attribute macro. `#[enum_range]` must be applied before any other derives because it changes the definition of the enum
/// Related attributes are: [Range]
#[proc_macro_attribute]
pub fn enum_range(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as DeriveInput);

    let repr = get_repr(&ast);

    let generated = match ast.data {
        Data::Enum(ref mut data_enum) => generate_enum_ranges(data_enum, &ast.ident, repr),
        _ => panic!("enum_range can only be applied to enum types"),
    };

    let result = quote! {
        #ast

        #generated
    };

    result.into()
}

/// Gets the first numerical representation associated to the enum
fn get_repr(ast: &DeriveInput) -> Option<Ident> {
    for attr in ast.attrs.iter() {
        if !attr.path().is_ident("repr") {
            continue;
        }

        let meta_list = attr.meta.require_list().unwrap();

        let reprs = meta_list
            .parse_args_with(Punctuated::<Ident, Token![,]>::parse_terminated)
            .unwrap();

        let regex = Regex::new(r"[uif]\d+").unwrap();

        return reprs
            .iter()
            .find(|repr| regex.is_match(&repr.to_string()))
            .cloned();
    }

    return None;
}

/// Generates the variants for each range defined on the enum
/// Changes the structure in place
fn generate_enum_ranges(
    data_enum: &mut DataEnum,
    enum_ident: &Ident,
    repr: Option<Ident>,
) -> proc_macro2::TokenStream {
    let mut ranges = VecDeque::new();

    // Find all ranges defined in the enum
    for (variant_index, variant) in data_enum.variants.iter_mut().enumerate() {
        if let Ok(range) = Range::from_variant(variant) {
            // extract "range" attribute
            let index = variant
                .attrs
                .iter()
                .position(|attr| attr.path().is_ident("range"));

            if let None = index {
                continue;
            }
            // remove the attribute after parsing it
            variant.attrs.remove(index.unwrap());

            ranges.push_back((variant_index, range))
        }
    }

    // No ranges, nothing to do
    if ranges.is_empty() {
        return quote!().into();
    }

    // This is the code for all the range_check generated
    let mut enum_impl = quote!();

    // Make the list of new variants
    let mut new_variants: Punctuated<Variant, Token![,]> = Punctuated::new();

    let mut current_range = ranges.pop_front();
    for (index, variant) in data_enum.variants.iter().enumerate() {

        if let Some((range_idx, range)) = &current_range {
            if index < *range_idx {
                // current variant is before the next variant-range to generate so we keep it as is
                new_variants.push(variant.clone());
            } else if index == *range_idx {
                // This variant needs to be replaced by a range
                for range_index in 0..range.end - range.start + 1 {
                    let range_value = range.start + range_index;
                    new_variants.push(Variant {
                        attrs: variant.attrs.clone(),
                        ident: generate_variant_ident(variant, range, range_index, range_value),
                        fields: Fields::Unit,
                        discriminant: Some((
                            syn::parse_str("=").unwrap(),
                            syn::parse_str(&range_value.to_string()).unwrap(),
                        )),
                    })
                }

                // Generate the associated range checker if we can
                let range_checker = generate_range_checker(enum_ident, &repr, range);
                enum_impl = quote! {
                    #enum_impl

                    #range_checker
                };

                current_range = ranges.pop_front();
            } else {
                // We can't be after the next range to generate because if we pass over a range we get the next one
                // which and they have the same ordering
                unreachable!()
            }
        } else {
            // We are done processing ranges, just add the final normal variants
            new_variants.push(variant.clone());
        }
    }

    // Change the enum definition in place
    data_enum.variants = new_variants;

    enum_impl
}

/// Generate a method for a range that checks if an enum variant is in it
fn generate_range_checker(
    enum_ident: &Ident,
    repr: &Option<Ident>,
    range: &Range,
) -> Option<proc_macro2::TokenStream> {
    if range.range_check.is_none() || repr.is_none() {
        return None;
    }

    let range_start = Literal::from_str(&range.start.to_string()).unwrap();
    let range_end = Literal::from_str(&range.end.to_string()).unwrap();
    let method_name = format_ident!("{}", range.range_check.as_ref().unwrap());

    return Some(quote! {
        impl #enum_ident {
            pub fn #method_name(self) -> bool {
                let value = self as #repr;
                value >= #range_start && value <= #range_end
            }
        }
    });
}

/// Generate the [Ident] for an enum variant in a range
fn generate_variant_ident(variant: &Variant, range: &Range, index: usize, value: usize) -> Ident {
    let format = range
        .format
        .clone()
        .unwrap_or_else(|| format!("{}{{}}", variant.ident));

    let ident_str = format
        .replace("{index}", &index.to_string())
        .replace("{value}", &value.to_string());
    Ident::new(&ident_str, Span::call_site())
}
