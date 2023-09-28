// Rust language amplification derive library providing multiple generic trait
// implementations, type wrappers, derive macros and other language enhancements
//
// Written in 2019-2020 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//     Elichai Turkel <elichai.turkel@gmail.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the MIT License
// along with this software.
// If not, see <https://opensource.org/licenses/MIT>.

use proc_macro2::{Span, TokenStream as TokenStream2};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute, Data, DataEnum, DataStruct, DataUnion, DeriveInput, Error, Field, Fields,
    FieldsNamed, FieldsUnnamed, Ident, Result, Type,
};

const NAME: &str = "from";
const EXAMPLE: &str = r#"#[from(::std::fmt::Error)]"#;

#[derive(Clone, PartialEq, Eq, Debug)]
enum InstructionEntity {
    Default,
    DefaultEnumFields {
        variant: Ident,
        fields: Vec<Ident>,
    },
    Unit {
        variant: Option<Ident>,
    },
    Named {
        variant: Option<Ident>,
        field: Ident,
        other: Vec<Ident>,
    },
    Unnamed {
        variant: Option<Ident>,
        index: usize,
        total: usize,
    },
}

impl InstructionEntity {
    pub fn with_fields(fields: &Fields, variant: Option<Ident>) -> Result<Self> {
        let res = match (fields.len(), variant, fields.clone(), fields.iter().next().cloned()) {
            (0, Some(v), ..) => InstructionEntity::Unit { variant: Some(v) },
            (_, variant, Fields::Unit, ..) => InstructionEntity::Unit { variant },
            (1, variant, Fields::Named(f), Some(Field { ident: Some(i), .. })) => {
                InstructionEntity::Named {
                    variant,
                    field: i.clone(),
                    other: f
                        .named
                        .iter()
                        .filter_map(|f| f.ident.clone())
                        .filter(|ident| ident != &i)
                        .collect(),
                }
            }
            (1, _, Fields::Named(_), ..) => {
                unreachable!("If we have named field, it will match previous option")
            }
            (_, Some(variant), Fields::Named(f), ..) => InstructionEntity::DefaultEnumFields {
                variant,
                fields: f.named.iter().filter_map(|f| f.ident.clone()).collect(),
            },
            (len, variant, Fields::Unnamed(_), ..) => InstructionEntity::Unnamed {
                variant,
                index: 0,
                total: len,
            },
            (_, None, ..) => InstructionEntity::Default,
        };
        Ok(res)
    }

    pub fn with_field(
        index: usize,
        total: usize,
        field: &Field,
        fields: &Fields,
        variant: Option<Ident>,
    ) -> Self {
        if let Some(ref ident) = field.ident {
            InstructionEntity::Named {
                variant,
                field: ident.clone(),
                other: fields
                    .iter()
                    .filter_map(|f| f.ident.clone())
                    .filter(|i| ident != i)
                    .collect(),
            }
        } else {
            InstructionEntity::Unnamed {
                variant,
                index,
                total,
            }
        }
    }

    pub fn into_token_stream2(self) -> TokenStream2 {
        match self {
            InstructionEntity::Default => quote! {
                Self::default()
            },
            InstructionEntity::Unit { variant } => {
                let var = variant.map_or(quote! {}, |v| quote! {:: #v});
                quote! { Self #var }
            }
            InstructionEntity::Named {
                variant: None,
                field,
                ..
            } => {
                quote! {
                    Self { #field: v.into(), ..Default::default() }
                }
            }
            InstructionEntity::Named {
                variant: Some(var),
                field,
                other,
            } => {
                quote! {
                    Self :: #var { #field: v.into(), #( #other: Default::default(), )* }
                }
            }
            InstructionEntity::Unnamed {
                variant,
                index,
                total,
            } => {
                let var = variant.map_or(quote! {}, |v| quote! {:: #v});
                let prefix = (0..index).fold(TokenStream2::new(), |mut stream, _| {
                    stream.extend(quote! {Default::default(),});
                    stream
                });
                let suffix = ((index + 1)..total).fold(TokenStream2::new(), |mut stream, _| {
                    stream.extend(quote! {Default::default(),});
                    stream
                });
                quote! {
                    Self #var ( #prefix v.into(), #suffix )
                }
            }
            InstructionEntity::DefaultEnumFields { variant, fields } => {
                quote! {
                    Self #variant { #( #fields: Default::default() )* }
                }
            }
        }
    }
}

#[derive(Clone)]
struct InstructionEntry(pub Type, pub InstructionEntity);

impl PartialEq for InstructionEntry {
    // Ugly way, but with current `syn` version no other way is possible
    fn eq(&self, other: &Self) -> bool {
        let l = &self.0;
        let r = &other.0;
        let a = quote! { #l };
        let b = quote! { #r };
        format!("{}", a) == format!("{}", b)
    }
}

impl InstructionEntry {
    pub fn with_type(ty: &Type, entity: &InstructionEntity) -> Self {
        Self(ty.clone(), entity.clone())
    }

    pub fn parse(
        fields: &Fields,
        attrs: &[Attribute],
        entity: InstructionEntity,
    ) -> Result<Vec<InstructionEntry>> {
        let mut list = Vec::<InstructionEntry>::new();
        for attr in attrs.iter().filter(|attr| attr.path.is_ident(NAME)) {
            // #[from]
            if attr.tokens.is_empty() {
                match (fields.len(), fields.iter().next()) {
                    (1, Some(field)) => list.push(InstructionEntry::with_type(&field.ty, &entity)),
                    _ => {
                        return Err(attr_err!(
                            attr,
                            "empty attribute is allowed only for entities with a single field; \
                             for multi-field entities specify the attribute right ahead of the \
                             target field"
                        ));
                    }
                }
            } else {
                list.push(InstructionEntry::with_type(&attr.parse_args()?, &entity));
            }
        }
        Ok(list)
    }
}

#[derive(Default)]
struct InstructionTable(Vec<InstructionEntry>);

impl InstructionTable {
    pub fn new() -> Self { Default::default() }

    pub fn parse(
        &mut self,
        fields: &Fields,
        attrs: &[Attribute],
        variant: Option<Ident>,
    ) -> Result<&Self> {
        let entity = InstructionEntity::with_fields(fields, variant.clone())?;
        self.extend(InstructionEntry::parse(fields, attrs, entity.clone())?)?;
        for (index, field) in fields.iter().enumerate() {
            let mut punctuated = Punctuated::new();
            punctuated.push_value(field.clone());
            self.extend(InstructionEntry::parse(
                &field.ident.as_ref().map_or(
                    Fields::Unnamed(FieldsUnnamed {
                        paren_token: Default::default(),
                        unnamed: punctuated.clone(),
                    }),
                    |_| {
                        Fields::Named(FieldsNamed {
                            brace_token: Default::default(),
                            named: punctuated,
                        })
                    },
                ),
                &field.attrs,
                InstructionEntity::with_field(index, fields.len(), field, fields, variant.clone()),
            )?)?;
        }
        if variant.is_none() && fields.len() == 1 && self.0.is_empty() {
            let field = fields
                .into_iter()
                .next()
                .expect("we know we have at least one item");
            self.push(InstructionEntry::with_type(&field.ty, &entity));
        }
        Ok(self)
    }

    fn push(&mut self, item: InstructionEntry) { self.0.push(item) }

    fn extend<T>(&mut self, list: T) -> Result<usize>
    where T: IntoIterator<Item = InstructionEntry> {
        let mut count = 0;
        for entry in list {
            self.0.iter().find(|e| *e == &entry).map_or(Ok(()), |_| {
                Err(Error::new(
                    Span::call_site(),
                    format!("Attribute `#[{}]`: repeated use of type `{}`", NAME, quote! {ty}),
                ))
            })?;
            self.0.push(entry);
            count += 1;
        }
        Ok(count)
    }

    pub fn into_token_stream2(self, input: &DeriveInput) -> TokenStream2 {
        let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
        let ident_name = &input.ident;

        self.0.into_iter().fold(TokenStream2::new(), |mut stream, InstructionEntry(from, entity)| {
            let convert = entity.into_token_stream2();
            stream.extend(quote! {
                #[automatically_derived]
                impl #impl_generics ::core::convert::From<#from> for #ident_name #ty_generics #where_clause {
                    fn from(v: #from) -> Self {
                        #convert
                    }
                }
            });
            stream
        })
    }
}

pub(crate) fn inner(input: DeriveInput) -> Result<TokenStream2> {
    match input.data {
        Data::Struct(ref data) => inner_struct(&input, data),
        Data::Enum(ref data) => inner_enum(&input, data),
        Data::Union(ref data) => inner_union(&input, data),
    }
}

fn inner_struct(input: &DeriveInput, data: &DataStruct) -> Result<TokenStream2> {
    let mut instructions = InstructionTable::new();
    instructions.parse(&data.fields, &input.attrs, None)?;
    Ok(instructions.into_token_stream2(input))
}

fn inner_enum(input: &DeriveInput, data: &DataEnum) -> Result<TokenStream2> {
    // Do not let top-level `from` on enums
    input
        .attrs
        .iter()
        .find(|attr| attr.path.is_ident(NAME))
        .map_or(Ok(()), |a| {
            Err(attr_err!(
                a,
                "top-level attribute is not allowed, use it for specific fields or variants"
            ))
        })?;

    let mut instructions = InstructionTable::new();
    for v in &data.variants {
        instructions.parse(&v.fields, &v.attrs, Some(v.ident.clone()))?;
    }
    Ok(instructions.into_token_stream2(input))
}

fn inner_union(input: &DeriveInput, data: &DataUnion) -> Result<TokenStream2> {
    let mut instructions = InstructionTable::new();
    instructions.parse(&Fields::Named(data.fields.clone()), &input.attrs, None)?;
    Ok(instructions.into_token_stream2(input))
}
