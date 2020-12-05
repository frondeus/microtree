use anyhow::Result;
use inflections::case::{to_lower_case, to_snake_case};
use itertools::Itertools;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::io::Write;
use std::{collections::BTreeMap, path::Path};
use syn::Ident;
use ungrammar::{Grammar, Rule, Token};

type Tokens = BTreeMap<String, String>;

#[derive(Debug, Deserialize)]
struct Config {
    tokens: Tokens,
    handwritten: Vec<String>,
}

impl Config {
    fn finish(self) -> Self {
        let Config {
            tokens,
            handwritten,
        } = self;

        let tokens: Tokens = tokens.into_iter().map(|(k, v)| (v, k)).collect();

        Self {
            tokens,
            handwritten,
        }
    }
}

pub fn codegen(
    config: impl AsRef<Path>,
    grammar: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<()> {
    let config = std::fs::read_to_string(config)?;
    let config: Config = serde_json::from_str(&config)?;
    let config = config.finish();

    let grammar = std::fs::read_to_string(grammar)?;
    let grammar: Grammar = grammar.parse().unwrap();

    let mut ast = lower(&config, &grammar);

    add_aliases(&mut ast);

    let output_path = output_path.as_ref();
    let generated_path = output_path.join("mod.rs");
    let handwritten_path = output_path.join("handwritten.rs");

    std::fs::create_dir_all(&output_path)?;

    let mut file = std::fs::File::create(&generated_path)?;

    writeln!(
        &mut file,
        "{}\n",
        quote! {
            #![allow(clippy::redundant_clone, clippy::wrong_self_convention)]
            #![allow(dead_code)]
            use microtree::{Red, Ast, AstBuilder, GreenBuilder, TokenBuilder, Green, AliasBuilder, IntoBuilder};
        }
    )?;

    if !config.handwritten.is_empty() {
        if !handwritten_path.exists() {
            let _ = std::fs::File::create(handwritten_path)?;
        }
        writeln!(
            &mut file,
            "{}\n",
            quote! {
                mod handwritten;
                pub use handwritten::*;
            }
        )?;
    }

    for token in &ast.tokens {
        if config.handwritten.contains(&token.name) {
            continue;
        }

        let token_name = format_ident!("{}", token.name);
        let tok = &token.token;

        let aliases = token
            .aliases
            .iter()
            .map(|alias| {
                let alias_ty = format_ident!("{}", alias);
                quote! (
                    impl IntoBuilder<#alias_ty> for TokenBuilder<#token_name> {
                        fn into_builder(self) -> AliasBuilder<Self, #alias_ty> {
                            AliasBuilder::new(#alias, self)
                        }
                    }
                )
            })
            .collect_vec();

        writeln!(
            &mut file,
            "{}\n",
            quote! (
                #[derive(Debug)]
                pub struct #token_name(Red);
                impl Ast for #token_name {
                    fn new(node: Red) -> Option<Self> {
                        if !node.is("token") {
                            return None;
                        }
                        let green = node.green();
                        let tok = green.as_token()?;
                        if tok.value != #tok {
                            return None;
                        }
                        Some(Self(node))
                    }

                    fn red(&self) -> Red {
                        self.0.clone()
                    }
                }
                impl #token_name {
                    pub fn build() -> TokenBuilder<#token_name> {
                        TokenBuilder::new(#tok)
                    }
                }
                #(#aliases)*
            )
        )?;
    }

    for node in ast.enums {
        if config.handwritten.contains(&node.name) {
            continue;
        }
        let enum_name = format_ident!("{}", node.name);
        let enum_variants = node
            .variants
            .iter()
            .map(|name| format_ident!("{}", name))
            .collect::<Vec<_>>();
        let as_variants = node
            .variants
            .iter()
            .map(|name| format_ident!("as_{}", to_lower_case(name)))
            .collect::<Vec<_>>();
        writeln!(
            &mut file,
            "{}\n",
            quote!(
                #[derive(Debug)]
                pub enum #enum_name {
                    #(#enum_variants(#enum_variants)),*
                }

                #(
                    impl From<#enum_variants> for #enum_name {
                        fn from(val: #enum_variants) -> Self {
                            Self::#enum_variants(val)
                        }
                    }
                )*

                impl Ast for #enum_name {
                    fn new(node: Red) -> Option<Self> {
                        None
                            #(
                                .or_else(|| #enum_variants::new(node.clone()).map(#enum_name::#enum_variants))
                            )*
                    }
                    fn red(&self) -> Red {
                        match &self {
                            #(
                            #enum_name::#enum_variants(node) => node.red()
                            ),*
                        }
                    }
                }

                impl #enum_name {
                    #(
                        pub fn #as_variants(self) -> Option<#enum_variants> {
                            match self {
                                Self::#enum_variants(val) => Some(val),
                                _ => None
                            }
                        }
                    )*
                }

            )
        )?;
    }

    for node in ast.nodes {
        if config.handwritten.contains(&node.name) {
            continue;
        }
        let node_name = format_ident!("{}", node.name);
        let node_builder_name = format_ident!("{}Builder", node.name);
        let node_name_lit = node.name;
        let fields = node
            .fields
            .iter()
            .map(|field| {
                let name = field.method_name();
                let ty = field.ty();

                if field.is_many() {
                    quote!(
                        pub fn #name(&self) -> impl Iterator<Item = #ty> + '_ {
                            self.0.children().filter_map(#ty::new)
                        }
                    )
                } else {
                    quote!(
                        pub fn #name(&self) -> Option<#ty> {
                            self.0.children().filter_map(#ty::new).next()
                        }
                    )
                }
            })
            .collect::<Vec<_>>();

        let build_args = node
            .fields
            .iter()
            .enumerate()
            .map(|(idx, field)| {
                let name = field.name();
                let ty = field.ty();
                let idx_ty = format_ident!("T{}", idx);
                match field.as_many() {
                    Some(Some(delit)) => {
                        let delit_name = format_ident!("{}", delit.name);
                        quote!(
                            #name: Vec<Box<dyn AstBuilder<T = #ty>>>,
                            #delit_name: #idx_ty
                        )
                    }
                    Some(None) => quote!(
                        #name: Vec<Box<dyn AstBuilder<T = #ty>>>
                    ),
                    None => quote!(#name: #idx_ty),
                }
            })
            .collect::<Vec<_>>();

        let build_field_names = node
            .fields
            .iter()
            .flat_map(|field| match field.as_many() {
                None | Some(None) => vec![field.name()],
                Some(Some(delit)) => vec![field.name(), format_ident!("{}", delit.name)],
            })
            .collect::<Vec<_>>();

        let build_fill = node
            .fields
            .iter()
            .map(|field| {
                let name = field.name();
                match field.as_many() {
                    None => quote!(#name: Some(#name)),
                    Some(None) => quote!(#name),
                    Some(Some(delit)) => {
                        let delit_name = format_ident!("{}", delit.name);
                        quote!(
                            #name,
                            #delit_name: Some(#delit_name)
                        )
                    }
                }
            })
            .collect::<Vec<_>>();

        let build_fields = node
            .fields
            .iter()
            .enumerate()
            .map(|(idx, field)| {
                let name = field.name();
                let ty = field.ty();
                let idx_ty = format_ident!("T{}", idx);
                match field.as_many() {
                    Some(None) => quote!(
                        #name: Vec<Box<dyn AstBuilder<T = #ty>>>
                    ),
                    Some(Some(delit)) => {
                        let delit_name = format_ident!("{}", delit.name);
                        quote!(
                            #name: Vec<Box<dyn AstBuilder<T = #ty>>>,
                            #delit_name: Option<#idx_ty>
                        )
                    }
                    None => quote!(#name: Option<#idx_ty>),
                }
            })
            .collect::<Vec<_>>();

        let build_generic_list = node
            .fields
            .iter()
            .enumerate()
            .filter_map(|(idx, field)| match field.as_many() {
                Some(None) => None,
                _ => Some(format_ident!("T{}", idx)),
            })
            .collect::<Vec<_>>();

        let build_where_list = node
            .fields
            .iter()
            .enumerate()
            .filter_map(|(idx, field)| {
                let ty = field.ty();
                let idx_ty = format_ident!("T{}", idx);
                match field.as_many() {
                    Some(Some(delit)) => {
                        let delit_ty = format_ident!("{}", delit.ty);
                        Some(quote!(
                            #idx_ty: AstBuilder<T = #delit_ty>
                        ))
                    }
                    Some(None) => None,
                    None => Some(quote!(#idx_ty: AstBuilder<T = #ty>)),
                }
            })
            .collect::<Vec<_>>();

        let build_generics = if build_generic_list.is_empty() {
            quote!()
        } else {
            quote!(<#(#build_generic_list),*>)
        };

        let where_generics = if build_generic_list.is_empty() {
            quote!()
        } else {
            quote!(
                where #( #build_where_list),*
            )
        };

        let children = node
            .fields
            .iter()
            .map(|field| {
                let name = field.name();
                //let ty = field.ty();
                match field.as_many() {
                    Some(Some(delit)) => {
                        let delit_name = format_ident!("{}", delit.name);
                        quote!(
                            .chain({
                                let delit = self.#delit_name.map(|it| it.build_green(builder));
                                self.#name.into_iter()
                                    .flat_map(|it|
                                              Some(it.build_boxed_green(builder))
                                              .into_iter()
                                              .chain(delit.clone().into_iter())
                                              .collect::<Vec<_>>()
                                          )
                                        .collect::<Vec<_>>()
                            })
                        )
                    }
                    Some(None) => quote!(
                        .chain({
                            self.#name.into_iter()
                                .map(|it| it.build_boxed_green(builder))
                                .collect::<Vec<_>>()
                        })
                    ),
                    None => quote!(
                        .chain(
                            self.#name.map(|it| it.build_green(builder))
                            .into_iter()
                        )
                    ),
                }
            })
            .collect::<Vec<_>>();

        let aliases = node.aliases.iter()
            .map(|alias| {
                let alias_ty = format_ident!("{}", alias);
                quote! (
                    impl #build_generics IntoBuilder<#alias_ty> for #node_builder_name #build_generics #where_generics {
                        fn into_builder(self) -> AliasBuilder<Self, #alias_ty> {
                            AliasBuilder::new(#alias, self)
                        }
                    }
                )
            })
            .collect_vec();

        writeln!(
            &mut file,
            "{}\n",
            quote! {
                #[derive(Debug)]
                pub struct #node_name(Red);
                impl Ast for #node_name {
                    fn new(node: Red) -> Option<Self> {
                        if !node.is(#node_name_lit) {
                            return None;
                        }
                        node.green().as_node()?;
                        Some(Self(node))
                    }

                    fn red(&self) -> Red {
                        self.0.clone()
                    }
                }

                impl #node_name {
                    #(#fields)*

                    pub fn build
                        #build_generics
                        () -> #node_builder_name #build_generics
                        #where_generics
                    {
                        Default::default()
                    }
                }

                pub struct #node_builder_name #build_generics
                        #where_generics
                {
                    #(#build_fields),*
                }

                impl #build_generics Default for #node_builder_name #build_generics
                        #where_generics
                {
                    fn default() -> Self {
                        Self {
                            #(#build_field_names: Default::default()),*
                        }
                    }
                }

                impl #build_generics #node_builder_name #build_generics
                        #where_generics
                {
                    pub fn fill(self, #(#build_args),*) -> Self {
                        Self {
                            #(#build_fill),*
                        }
                    }
                }

                impl #build_generics AstBuilder for #node_builder_name #build_generics
                        #where_generics
                {
                    type T = #node_name;
                    fn build(self, builder: &mut GreenBuilder) -> #node_name {
                        let green = AstBuilder::build_green(self, builder);
                        #node_name::new(Red::root(green)).unwrap()
                    }
                    fn build_boxed_green(self: Box<Self>, builder: &mut GreenBuilder) -> Green {
                        AstBuilder::build_green(*self, builder)
                    }
                    fn build_green(self, builder: &mut GreenBuilder) -> Green {
                              let children = None.into_iter()
                                #(#children)*
                                .collect();
                        //let children = vec![];
                        builder.node(#node_name_lit, move |_| children)
                    }
                }

                #(#aliases)*
            }
        )?;
    }

    duct::cmd!("rustfmt", generated_path).run()?;

    Ok(())
}

#[derive(Default, Debug)]
struct AstSrc {
    //tokens: Tokens,
    tokens: Vec<AstTokenSrc>,
    nodes: Vec<AstNodeSrc>,
    enums: Vec<AstEnumSrc>,
}

#[derive(Debug)]
struct AstTokenSrc {
    name: String,
    token: String,
    aliases: Vec<String>,
}

#[derive(Debug)]
struct AstNodeSrc {
    name: String,
    fields: Vec<AstField>,
    aliases: Vec<String>,
}

#[derive(Debug)]
enum AstField {
    Token {
        ty: String,
        name: String,
    },
    Node {
        name: String,
        ty: String,
        cardinality: Cardinality,
    },
}

impl AstField {
    fn method_name(&self) -> Ident {
        match self {
            AstField::Token { name, .. } => format_ident!("{}_token", name),
            AstField::Node { name, .. } => format_ident!("{}", name),
        }
    }

    fn name(&self) -> Ident {
        match self {
            AstField::Token { name, .. } | AstField::Node { name, .. } => format_ident!("{}", name),
        }
    }

    fn ty(&self) -> Ident {
        match self {
            AstField::Token { ty, .. } | AstField::Node { ty, .. } => format_ident!("{}", ty),
        }
    }

    fn as_many(&self) -> Option<Option<&Delimiter>> {
        match self {
            Self::Node {
                cardinality: Cardinality::Many(delit),
                ..
            } => Some(delit.as_ref()),
            _ => None,
        }
    }

    fn is_many(&self) -> bool {
        matches!(self, Self::Node { cardinality: Cardinality::Many(_), .. })
    }
}

#[derive(Debug)]
struct Delimiter {
    ty: String,
    name: String,
}

#[derive(Debug)]
enum Cardinality {
    Optional,
    Many(Option<Delimiter>),
}

#[derive(Debug)]
struct AstEnumSrc {
    name: String,
    variants: Vec<String>,
}

fn lower(config: &Config, grammar: &Grammar) -> AstSrc {
    let mut res = AstSrc::default();
    res.tokens = config
        .tokens
        .iter()
        .map(|(tok, tok_name)| AstTokenSrc {
            name: tok_name.clone(),
            aliases: Default::default(),
            token: tok.clone(),
        })
        .collect();

    for node in grammar.iter() {
        let node = &grammar[node];
        let name = node.name.clone();
        match lower_enum(config, &grammar, &node.rule) {
            Some(variants) => {
                res.enums.push(AstEnumSrc { name, variants });
            }
            None => {
                let mut fields = Vec::new();
                lower_rule(&mut fields, grammar, config, None, &node.rule);
                res.nodes.push(AstNodeSrc {
                    name,
                    fields,
                    aliases: Default::default(),
                });
            }
        };
    }

    res
}

fn lower_comma_list(
    acc: &mut Vec<AstField>,
    grammar: &Grammar,
    config: &Config,
    label: Option<&String>,
    rule: &Rule,
) -> bool {
    let rule = match rule {
        Rule::Seq(it) => it,
        _ => return false,
    };

    let (node, repeat, trailing_comma) = match rule.as_slice() {
        [Rule::Node(node), Rule::Rep(repeat), Rule::Opt(trailing_comma)] => {
            (node, repeat, Some(trailing_comma))
        }
        [Rule::Node(node), Rule::Rep(repeat)] => (node, repeat, None),
        _ => return false,
    };

    let trailing_comma = match trailing_comma {
        Some(t_c) => match &**t_c {
            Rule::Token(t_c) => Some(t_c),
            _ => return false,
        },
        None => None,
    };

    let repeat = match &**repeat {
        Rule::Seq(it) => it,
        _ => return false,
    };

    let is_like_trailing_comma = |tok: &Token| -> bool {
        match trailing_comma {
            None => true,
            Some(t_c) => tok == t_c,
        }
    };

    let delimiter = match repeat.as_slice() {
        [Rule::Token(comma), Rule::Node(n)] if is_like_trailing_comma(comma) && n == node => comma,
        _ => return false,
    };

    let delimiter_token = &grammar[*delimiter].name;
    let delimiter_ty = config.tokens[delimiter_token].clone();
    let delimiter_name = to_snake_case(&delimiter_ty);
    let delimiter = Delimiter {
        ty: delimiter_ty,
        name: delimiter_name,
    };

    let ty = grammar[*node].name.clone();
    let name = label.cloned().unwrap_or_else(|| to_snake_case(&ty));
    acc.push(AstField::Node {
        name,
        ty,
        cardinality: Cardinality::Many(Some(delimiter)),
    });
    true
}

fn lower_rule(
    acc: &mut Vec<AstField>,
    grammar: &Grammar,
    config: &Config,
    label: Option<&String>,
    rule: &Rule,
) {
    if lower_comma_list(acc, grammar, config, label, rule) {
        return;
    }

    match rule {
        Rule::Node(node) => {
            let ty = grammar[*node].name.clone();
            let name = label.cloned().unwrap_or_else(|| to_snake_case(&ty));
            acc.push(AstField::Node {
                name,
                ty,
                cardinality: Cardinality::Optional,
            });
        }
        Rule::Token(token) => {
            let token = grammar[*token].name.clone();
            let ty = config.tokens[&token].clone();
            let name = label.cloned().unwrap_or_else(|| to_snake_case(&ty));

            acc.push(AstField::Token { name, ty });
        }
        Rule::Seq(rules) | Rule::Alt(rules) => {
            for rule in rules {
                lower_rule(acc, grammar, config, label, rule);
            }
        }
        Rule::Rep(inner) => match &**inner {
            Rule::Node(node) => {
                let ty = grammar[*node].name.clone();
                let name = label.cloned().unwrap_or_else(|| to_snake_case(&ty));
                acc.push(AstField::Node {
                    name,
                    ty,
                    cardinality: Cardinality::Many(None),
                });
            }
            _ => todo!("REP: {:?}", inner),
        },
        Rule::Labeled { label: l, rule } => {
            lower_rule(acc, grammar, config, Some(l), rule);
        }
        Rule::Opt(rule) => lower_rule(acc, grammar, config, label, rule),
    }
}

fn lower_enum(config: &Config, grammar: &Grammar, rule: &Rule) -> Option<Vec<String>> {
    let alt = match rule {
        Rule::Alt(it) => it,
        _ => return None,
    };

    alt.iter()
        .map(|alt| match alt {
            Rule::Node(it) => Some(grammar[*it].name.clone()),
            Rule::Token(it) => {
                let token = &grammar[*it].name;
                Some(config.tokens[token].clone())
            }
            _ => None,
        })
        .collect()
}

fn add_aliases(ast: &mut AstSrc) {
    let enum_variants = ast
        .enums
        .iter()
        .flat_map(|e| e.variants.iter().map(move |v| (&e.name, v)))
        .unique()
        .collect_vec();

    if enum_variants.is_empty() {
        return;
    }

    for node in ast.nodes.iter_mut() {
        if let Some(e) = enum_variants
            .iter()
            .find(|(_, v)| v == &&node.name)
            .map(|(e, _)| e)
        {
            node.aliases.push((*e).clone());
        }
    }

    for token in ast.tokens.iter_mut() {
        if let Some(e) = enum_variants
            .iter()
            .find(|(_, v)| v == &&token.name)
            .map(|(e, _)| e)
        {
            token.aliases.push((*e).clone());
        }
    }
}
