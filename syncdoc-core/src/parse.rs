//! Declarative parsing types using unsynn

use unsynn::*;

/// Parses tokens until `C` is found on the current token tree level.
pub type VerbatimUntil<C> = Many<Cons<Except<C>, AngleTokenTree>>;

keyword! {
    /// The "path" keyword
    pub KPath = "path";
    /// The "name" keyword
    pub KName = "name";
    /// The "cfg_attr" keyword
    pub KCfgAttr = "cfg_attr";
    /// The "fn" keyword
    pub KFn = "fn";
    /// The "pub" keyword
    pub KPub = "pub";
    /// The "async" keyword
    pub KAsync = "async";
    /// The "unsafe" keyword
    pub KUnsafe = "unsafe";
    /// The "extern" keyword
    pub KExtern = "extern";
    /// The "const" keyword
    pub KConst = "const";
    /// The "where" keyword
    pub KWhere = "where";
    /// The "impl" keyword
    pub KImpl = "impl";
    /// The "for" keyword
    pub KFor = "for";
    /// The "mod" keyword
    pub KMod = "mod";
    /// The "trait" keyword
    pub KTrait = "trait";
    /// The "crate" keyword
    pub KCrate = "crate";
    /// The "super" keyword
    pub KSuper = "super";
    /// The "self" keyword
    pub KSelf = "self";
    /// The "mut" keyword
    pub KMut = "mut";
    /// The "enum" keyword
    pub KEnum = "enum";
    /// The "struct" keyword
    pub KStruct = "struct";
    /// The "type" keyword
    pub KType = "type";
    /// The "static" keyword
    pub KStatic = "static";
}

operator! {
    /// The "=" operator
    pub Eq = "=";
    /// The "&" operator
    pub And = "&";
}

unsynn! {
    /// Parses either a `TokenTree` or `<...>` grouping
    #[derive(Clone)]
    pub struct AngleTokenTree(
        pub Either<Cons<Lt, Vec<Cons<Except<Gt>, AngleTokenTree>>, Gt>, TokenTree>,
    );

    /// Declarative syncdoc arguments structure
    pub struct SyncDocInner {
        /// Comma-delimited list of arguments
        pub args: Option<CommaDelimitedVec<SyncDocArg>>,
    }

    /// Single syncdoc argument
    pub enum SyncDocArg {
        /// path = "docs"
        Path(PathArg),
        /// name = "custom"
        Name(NameArg),
        /// cfg_attr = "doc"
        CfgAttr(CfgAttrArg),
    }

    /// Path argument: path = "docs"
    pub struct PathArg {
        pub _path: KPath,
        pub _eq: Eq,
        pub value: LiteralString,
    }

    /// Name argument: name = "custom"
    pub struct NameArg {
        pub _name: KName,
        pub _eq: Eq,
        pub value: LiteralString,
    }

    /// Name argument: name = "custom"
    pub struct CfgAttrArg {
        pub _cfg_attr: KCfgAttr,
        pub _eq: Eq,
        pub value: LiteralString,
    }

    /// Complete function signature
    #[derive(Clone)]
    pub struct FnSig {
        /// Optional attributes (#[...])
        pub attributes: Option<Many<Attribute>>,
        /// Optional visibility (pub, pub(crate), etc.)
        pub visibility: Option<Visibility>,
        /// Optional const modifier
        pub const_kw: Option<KConst>,
        /// Optional async modifier
        pub async_kw: Option<KAsync>,
        /// Optional unsafe modifier
        pub unsafe_kw: Option<KUnsafe>,
        /// Optional extern with optional ABI
        pub extern_kw: Option<ExternSpec>,
        /// The "fn" keyword
        pub _fn: KFn,
        /// Function name
        pub name: Ident,
        /// Optional generic parameters
        pub generics: Option<Generics>,
        /// Parameters in parentheses
        pub params: ParenthesisGroupContaining<Option<CommaDelimitedVec<FnParam>>>,
        /// Optional return type
        pub return_type: Option<ReturnType>,
        /// Optional where clause
        pub where_clause: Option<WhereClauses>,
        pub body: BraceGroup,
    }

    /// (Outer) Attribute like #[derive(Debug)]
    #[derive(Clone)]
    pub struct Attribute {
        /// Hash symbol
        pub _hash: Pound,
        /// Attribute content
        pub content: BracketGroup,
    }

    /// Inner attribute like #![forbid(unsafe_code)]
    #[derive(Clone)]
    pub struct InnerAttribute {
        /// Hash symbol
        pub _hash: Pound,
        /// Bang symbol
        pub _bang: Bang,
        /// Attribute content
        pub content: BracketGroup,
    }

    /// Either an inner or outer attribute
    pub enum AnyAttribute {
        Inner(InnerAttribute),
        Outer(Attribute),
    }

    /// Extern specification with optional ABI
    #[derive(Clone)]
    pub enum ExternSpec {
        /// "extern" with ABI string like extern "C"
        WithAbi(ExternWithAbi),
        /// Just "extern"
        Bare(KExtern),
    }

    /// Extern with ABI string
    #[derive(Clone)]
    pub struct ExternWithAbi {
        /// The "extern" keyword
        pub _extern: KExtern,
        /// The ABI string
        pub abi: LiteralString,
    }

    /// Simple visibility parsing
    #[derive(Clone)]
    pub enum Visibility {
        /// "pub(crate)", "pub(super)", etc.
        Restricted(RestrictedVis),
        /// Just "pub"
        Public(KPub),
    }

    /// Restricted visibility like pub(crate)
    #[derive(Clone)]
    pub struct RestrictedVis {
        /// The "pub" keyword
        pub _pub: KPub,
        /// The parentheses with content
        pub restriction: ParenthesisGroup,
    }

    /// Simple generics (treat as opaque for now)
    #[derive(Clone)]
    pub struct Generics {
        /// Opening
        pub _lt: Lt,
        /// Everything until closing > (opaque)
        pub content: Many<Cons<Except<Gt>, TokenTree>>,
        /// Closing >
        pub _gt: Gt,
    }

    /// Return type: -> Type
    #[derive(Clone)]
    pub struct ReturnType {
        /// Arrow
        pub _arrow: RArrow,
        /// Everything until brace (opaque)
        pub return_type: VerbatimUntil<Either<BraceGroup, KWhere, Semicolon>>,
    }

    /// Represents a single predicate within a `where` clause.
    #[derive(Clone)]
    pub struct WhereClause {
        /// The type or lifetime being constrained (e.g., `T` or `'a`).
        pub _pred: VerbatimUntil<Colon>,
        /// The colon separating the constrained item and its bounds.
        pub _colon: Colon,
        /// The bounds applied to the type or lifetime (e.g., `Trait` or `'b`).
        pub bounds: VerbatimUntil<Either<Comma, Semicolon, BraceGroup>>,
    }

    /// Where clauses: where T: Trait, U: Send
    #[derive(Clone)]
    pub struct WhereClauses {
        /// The `where` keyword.
        pub _kw_where: KWhere,
        /// The comma-delimited list of where clause predicates.
        pub clauses: CommaDelimitedVec<WhereClausePredicate>,
    }

    /// Single where clause predicate: T: Trait
    #[derive(Clone)]
    pub struct WhereClausePredicate {
        /// The type being constrained (e.g., `T`)
        pub pred: VerbatimUntil<Colon>,
        /// The colon
        pub _colon: Colon,
        /// The bounds (e.g., `Trait`)
        pub bounds: VerbatimUntil<Either<Comma, BraceGroup>>,
    }

    /// Top-level item that can appear in a module
    #[derive(Clone)]
    pub enum ModuleItem {
		/// A trait method signature (no body)
		TraitMethod(TraitMethodSig),
        /// A function definition
        Function(FnSig),
        /// An impl block
        ImplBlock(ImplBlockSig),
        /// A module definition
        Module(ModuleSig),
        /// A trait definition
        Trait(TraitSig),
        /// An enum definition
        Enum(EnumSig),
        /// A struct definition
        Struct(StructSig),
        /// A type alias
        TypeAlias(TypeAliasSig),
        /// A constant
        Const(ConstSig),
        /// A static
        Static(StaticSig),
        /// Any other item (use, extern crate, etc.)
        Other(TokenTree),
    }

    /// impl Type { ... } block
    #[derive(Clone)]
    pub struct ImplBlockSig {
        /// Optional attributes
        pub attributes: Option<Many<Attribute>>,
        /// "impl" keyword
        pub _impl: KImpl,
        /// Optional generic parameters
        pub generics: Option<Generics>,
        /// Type being implemented (opaque for now)
        pub target_type: Many<Cons<Except<Either<KFor, BraceGroup>>, TokenTree>>,
        /// Optional "for Trait" part
        pub for_trait: Option<Cons<KFor, Many<Cons<Except<BraceGroup>, TokenTree>>>>,
        /// Optional where clause
        pub where_clause: Option<WhereClauses>,
        /// Parsed impl block contents
        pub items: BraceGroupContaining<ModuleContent>,
    }

    /// mod name { ... } block
    #[derive(Clone)]
    pub struct ModuleSig {
        /// Optional attributes
        pub attributes: Option<Many<Attribute>>,
        /// Optional visibility
        pub visibility: Option<Visibility>,
        /// "mod" keyword
        pub _mod: KMod,
        /// Module name
        pub name: Ident,
        /// Parsed module contents
        pub items: BraceGroupContaining<ModuleContent>,
    }

    /// trait Name { ... } block
    #[derive(Clone)]
    pub struct TraitSig {
        /// Optional attributes
        pub attributes: Option<Many<Attribute>>,
        /// Optional visibility
        pub visibility: Option<Visibility>,
        /// Optional unsafe
        pub unsafe_kw: Option<KUnsafe>,
        /// "trait" keyword
        pub _trait: KTrait,
        /// Trait name
        pub name: Ident,
        /// Optional generic parameters
        pub generics: Option<Generics>,
        /// Optional trait bounds
        pub bounds: Option<Cons<Colon, Many<Cons<Except<Either<KWhere, BraceGroup>>, TokenTree>>>>,
        /// Optional where clause
        pub where_clause: Option<WhereClauses>,
        /// Parsed trait body
        pub items: BraceGroupContaining<ModuleContent>,
    }

	/// Trait method signature (no body)
	#[derive(Clone)]
	pub struct TraitMethodSig {
		/// Optional attributes
		pub attributes: Option<Many<Attribute>>,
		/// Optional const modifier
		pub const_kw: Option<KConst>,
		/// Optional async modifier
		pub async_kw: Option<KAsync>,
		/// Optional unsafe modifier
		pub unsafe_kw: Option<KUnsafe>,
		/// Optional extern with optional ABI
		pub extern_kw: Option<ExternSpec>,
		/// The "fn" keyword
		pub _fn: KFn,
		/// Method name
		pub name: Ident,
		/// Optional generic parameters
		pub generics: Option<Generics>,
		/// Parameters in parentheses
		pub params: ParenthesisGroupContaining<Option<CommaDelimitedVec<FnParam>>>,
		/// Optional return type
		pub return_type: Option<ReturnType>,
		/// Optional where clause
		pub where_clause: Option<WhereClauses>,
		/// Semicolon (trait methods end with ;, not {})
		pub _semi: Semicolon,
	}

    /// enum Name { ... } block
    #[derive(Clone)]
    pub struct EnumSig {
        /// Optional attributes
        pub attributes: Option<Many<Attribute>>,
        /// Optional visibility
        pub visibility: Option<Visibility>,
        /// "enum" keyword
        pub _enum: KEnum,
        /// Enum name
        pub name: Ident,
        /// Optional generic parameters
        pub generics: Option<Generics>,
        /// Optional where clause
        pub where_clause: Option<WhereClauses>,
        /// Parsed enum variants
        pub variants: BraceGroupContaining<Option<CommaDelimitedVec<EnumVariant>>>,
    }

    /// struct Name { ... } or struct Name;
    #[derive(Clone)]
    pub struct StructSig {
        /// Optional attributes
        pub attributes: Option<Many<Attribute>>,
        /// Optional visibility
        pub visibility: Option<Visibility>,
        /// "struct" keyword
        pub _struct: KStruct,
        /// Struct name
        pub name: Ident,
        /// Optional generic parameters
        pub generics: Option<Generics>,
        /// Optional where clause
        pub where_clause: Option<WhereClauses>,
        /// Struct body (could be brace group, tuple, or unit)
        pub body: StructBody,
    }

    /// Struct body variants
    #[derive(Clone)]
    pub enum StructBody {
        /// Named fields with parsed field list
        Named(BraceGroupContaining<Option<CommaDelimitedVec<StructField>>>),
        /// Tuple fields: (Type, Type)
        Tuple(Cons<ParenthesisGroup, Semicolon>),
        /// Unit struct: ;
        Unit(Semicolon),
    }

    /// Named struct field: pub name: Type
    #[derive(Clone)]
    pub struct StructField {
        /// Optional attributes
        pub attributes: Option<Many<Attribute>>,
        /// Optional visibility
        pub visibility: Option<Visibility>,
        /// Field name
        pub name: Ident,
        /// Colon
        pub _colon: Colon,
        /// Field type (everything until comma or brace closing)
        pub field_type: VerbatimUntil<Either<Comma, BraceGroup>>,
    }

    /// type Alias = Type;
    #[derive(Clone)]
    pub struct TypeAliasSig {
        /// Optional attributes
        pub attributes: Option<Many<Attribute>>,
        /// Optional visibility
        pub visibility: Option<Visibility>,
        /// "type" keyword
        pub _type: KType,
        /// Alias name
        pub name: Ident,
        /// Optional generic parameters
        pub generics: Option<Generics>,
        /// Equals sign
        pub _eq: Eq,
        /// Target type (everything until semicolon)
        pub target: VerbatimUntil<Semicolon>,
        /// Semicolon
        pub _semi: Semicolon,
    }

    /// const NAME: Type = value;
    #[derive(Clone)]
    pub struct ConstSig {
        /// Optional attributes
        pub attributes: Option<Many<Attribute>>,
        /// Optional visibility
        pub visibility: Option<Visibility>,
        /// "const" keyword
        pub _const: KConst,
        /// Constant name
        pub name: Ident,
        /// Colon
        pub _colon: Colon,
        /// Type (everything until equals)
        pub const_type: VerbatimUntil<Eq>,
        /// Equals sign
        pub _eq: Eq,
        /// Value (everything until semicolon)
        pub value: VerbatimUntil<Semicolon>,
        /// Semicolon
        pub _semi: Semicolon,
    }

    /// static NAME: Type = value;
    #[derive(Clone)]
    pub struct StaticSig {
        /// Optional attributes
        pub attributes: Option<Many<Attribute>>,
        /// Optional visibility
        pub visibility: Option<Visibility>,
        /// Optional mut keyword
        pub mut_kw: Option<KMut>,
        /// "static" keyword
        pub _static: KStatic,
        /// Static name
        pub name: Ident,
        /// Colon
        pub _colon: Colon,
        /// Type (everything until equals)
        pub static_type: VerbatimUntil<Eq>,
        /// Equals sign
        pub _eq: Eq,
        /// Value (everything until semicolon)
        pub value: VerbatimUntil<Semicolon>,
        /// Semicolon
        pub _semi: Semicolon,
    }

    /// Single enum variant
    #[derive(Clone)]
    pub struct EnumVariant {
        /// Optional attributes
        pub attributes: Option<Many<Attribute>>,
        /// Variant name
        pub name: Ident,
        /// Optional variant data (fields or discriminant)
        pub data: Option<EnumVariantData>,
    }

    /// Enum variant data
    #[derive(Clone)]
    pub enum EnumVariantData {
        /// Tuple variant: (Type, Type)
        Tuple(ParenthesisGroup),
        /// Struct variant: { field: Type }
        Struct(BraceGroupContaining<Option<CommaDelimitedVec<StructField>>>),
        /// Discriminant: = value
        Discriminant(Cons<Eq, VerbatimUntil<Either<Comma, BraceGroup>>>),
    }

    /// A complete module/file content
    #[derive(Clone)]
    pub struct ModuleContent {
        /// Inner attributes at the top of the module (#![...])
        pub inner_attrs: Option<Many<InnerAttribute>>,
        /// All items in the module
        pub items: Many<ModuleItem>,
    }

    /// Function parameter: name: Type or self variants
    #[derive(Clone)]
    pub enum FnParam {
        /// self parameter
        SelfParam(SelfParam),
        /// Regular parameter: name: Type
        Named(NamedParam),
        /// Pattern parameter: (a, b): (i32, i32)
        Pattern(PatternParam),
    }

    /// self, &self, &mut self, mut self
    #[derive(Clone)]
    pub enum SelfParam {
        /// self
        Value(KSelf),
        /// &self
        Ref(Cons<And, KSelf>),
        /// &mut self
        RefMut(Cons<And, Cons<KMut, KSelf>>),
        /// mut self
        Mut(Cons<KMut, KSelf>),
    }

    /// name: Type parameter
    #[derive(Clone)]
    pub struct NamedParam {
        /// Optional mut keyword
        pub mut_kw: Option<KMut>,
        /// Parameter name
        pub name: Ident,
        /// Colon
        pub _colon: Colon,
        /// Parameter type (opaque for now)
        pub param_type: VerbatimUntil<Comma>,
    }

    /// Pattern parameter like (a, b): (i32, i32) or mut (x, y): Point
    #[derive(Clone)]
    pub struct PatternParam {
        /// Optional mut keyword
        pub mut_kw: Option<KMut>,
        /// Pattern (everything before colon, could be tuple, struct pattern, etc.)
        pub pattern: Pattern,
        /// Colon
        pub _colon: Colon,
        /// Parameter type
        pub param_type: VerbatimUntil<Either<Comma, ParenthesisGroup>>,
    }

    /// Different types of patterns
    #[derive(Clone)]
    pub enum Pattern {
        /// Simple identifier: value
        Ident(Ident),
        /// Tuple pattern: (a, b, c)
        Tuple(TuplePattern),
        /// Other patterns (fallback)
        Other(VerbatimUntil<Colon>),
    }

    /// Tuple destructuring pattern: (a, b, c)
    #[derive(Clone)]
    pub struct TuplePattern {
        /// Parentheses containing comma-separated identifiers
        pub fields: ParenthesisGroupContaining<Option<CommaDelimitedVec<PatternField>>>,
    }

    /// Field in a pattern
    #[derive(Clone)]
    pub enum PatternField {
        /// Simple identifier
        Ident(Ident),
        /// Nested pattern (recursive)
        Nested(Pattern),
    }
}

// Implement ToTokens for quote! compatibility
impl quote::ToTokens for FnSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // Add attributes
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }

        // Add visibility
        if let Some(vis) = &self.visibility {
            quote::ToTokens::to_tokens(vis, tokens);
        }

        // Add const keyword
        if let Some(const_kw) = &self.const_kw {
            unsynn::ToTokens::to_tokens(const_kw, tokens);
        }

        // Add async keyword
        if let Some(async_kw) = &self.async_kw {
            unsynn::ToTokens::to_tokens(async_kw, tokens);
        }

        // Add unsafe keyword
        if let Some(unsafe_kw) = &self.unsafe_kw {
            unsynn::ToTokens::to_tokens(unsafe_kw, tokens);
        }

        // Add extern specification
        if let Some(extern_kw) = &self.extern_kw {
            unsynn::ToTokens::to_tokens(extern_kw, tokens);
        }

        // Add fn keyword and the rest
        unsynn::ToTokens::to_tokens(&self._fn, tokens);
        quote::ToTokens::to_tokens(&self.name, tokens);

        if let Some(generics) = &self.generics {
            unsynn::ToTokens::to_tokens(generics, tokens);
        }

        unsynn::ToTokens::to_tokens(&self.params, tokens);

        if let Some(ret_type) = &self.return_type {
            unsynn::ToTokens::to_tokens(ret_type, tokens);
        }

        if let Some(where_clause) = &self.where_clause {
            unsynn::ToTokens::to_tokens(where_clause, tokens);
        }

        unsynn::ToTokens::to_tokens(&self.body, tokens);
    }
}

impl quote::ToTokens for TraitMethodSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // Add attributes
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }

        // Add const keyword
        if let Some(const_kw) = &self.const_kw {
            unsynn::ToTokens::to_tokens(const_kw, tokens);
        }

        // Add async keyword
        if let Some(async_kw) = &self.async_kw {
            unsynn::ToTokens::to_tokens(async_kw, tokens);
        }

        // Add unsafe keyword
        if let Some(unsafe_kw) = &self.unsafe_kw {
            unsynn::ToTokens::to_tokens(unsafe_kw, tokens);
        }

        // Add extern specification
        if let Some(extern_kw) = &self.extern_kw {
            unsynn::ToTokens::to_tokens(extern_kw, tokens);
        }

        // Add fn keyword and the rest
        unsynn::ToTokens::to_tokens(&self._fn, tokens);
        quote::ToTokens::to_tokens(&self.name, tokens);

        if let Some(generics) = &self.generics {
            unsynn::ToTokens::to_tokens(generics, tokens);
        }

        unsynn::ToTokens::to_tokens(&self.params, tokens);

        if let Some(ret_type) = &self.return_type {
            unsynn::ToTokens::to_tokens(ret_type, tokens);
        }

        if let Some(where_clause) = &self.where_clause {
            unsynn::ToTokens::to_tokens(where_clause, tokens);
        }

        unsynn::ToTokens::to_tokens(&self._semi, tokens);
    }
}

impl quote::ToTokens for FnParam {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FnParam::SelfParam(self_param) => quote::ToTokens::to_tokens(self_param, tokens),
            FnParam::Named(named) => quote::ToTokens::to_tokens(named, tokens),
            FnParam::Pattern(pattern) => quote::ToTokens::to_tokens(pattern, tokens),
        }
    }
}

impl quote::ToTokens for SelfParam {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            SelfParam::Value(self_kw) => unsynn::ToTokens::to_tokens(self_kw, tokens),
            SelfParam::Ref(ref_self) => unsynn::ToTokens::to_tokens(ref_self, tokens),
            SelfParam::RefMut(ref_mut_self) => unsynn::ToTokens::to_tokens(ref_mut_self, tokens),
            SelfParam::Mut(mut_self) => unsynn::ToTokens::to_tokens(mut_self, tokens),
        }
    }
}

impl quote::ToTokens for NamedParam {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(mut_kw) = &self.mut_kw {
            unsynn::ToTokens::to_tokens(mut_kw, tokens);
        }
        quote::ToTokens::to_tokens(&self.name, tokens);
        unsynn::ToTokens::to_tokens(&self._colon, tokens);
        unsynn::ToTokens::to_tokens(&self.param_type, tokens);
    }
}

impl quote::ToTokens for PatternParam {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(mut_kw) = &self.mut_kw {
            unsynn::ToTokens::to_tokens(mut_kw, tokens);
        }
        unsynn::ToTokens::to_tokens(&self.pattern, tokens);
        unsynn::ToTokens::to_tokens(&self._colon, tokens);
        unsynn::ToTokens::to_tokens(&self.param_type, tokens);
    }
}

impl quote::ToTokens for Pattern {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Pattern::Tuple(tuple) => quote::ToTokens::to_tokens(tuple, tokens),
            Pattern::Ident(ident) => quote::ToTokens::to_tokens(ident, tokens),
            Pattern::Other(other) => unsynn::ToTokens::to_tokens(other, tokens),
        }
    }
}

impl quote::ToTokens for TuplePattern {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self.fields, tokens);
    }
}

impl quote::ToTokens for PatternField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            PatternField::Ident(ident) => quote::ToTokens::to_tokens(ident, tokens),
            PatternField::Nested(pattern) => quote::ToTokens::to_tokens(pattern, tokens),
        }
    }
}

impl quote::ToTokens for Visibility {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Visibility::Public(pub_kw) => unsynn::ToTokens::to_tokens(pub_kw, tokens),
            Visibility::Restricted(restricted) => unsynn::ToTokens::to_tokens(restricted, tokens),
        }
    }
}

impl quote::ToTokens for ExternSpec {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ExternSpec::WithAbi(with_abi) => unsynn::ToTokens::to_tokens(with_abi, tokens),
            ExternSpec::Bare(extern_kw) => unsynn::ToTokens::to_tokens(extern_kw, tokens),
        }
    }
}

impl quote::ToTokens for ReturnType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._arrow, tokens);
        unsynn::ToTokens::to_tokens(&self.return_type, tokens);
    }
}

impl quote::ToTokens for Generics {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._lt, tokens);
        unsynn::ToTokens::to_tokens(&self.content, tokens);
        unsynn::ToTokens::to_tokens(&self._gt, tokens);
    }
}

impl quote::ToTokens for WhereClause {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._pred, tokens);
        unsynn::ToTokens::to_tokens(&self._colon, tokens);
        unsynn::ToTokens::to_tokens(&self.bounds, tokens);
    }
}

impl quote::ToTokens for WhereClauses {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._kw_where, tokens);
        unsynn::ToTokens::to_tokens(&self.clauses, tokens);
    }
}

impl quote::ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._hash, tokens);
        unsynn::ToTokens::to_tokens(&self.content, tokens);
    }
}

impl quote::ToTokens for InnerAttribute {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._hash, tokens);
        unsynn::ToTokens::to_tokens(&self._bang, tokens);
        unsynn::ToTokens::to_tokens(&self.content, tokens);
    }
}

impl quote::ToTokens for AnyAttribute {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            AnyAttribute::Inner(inner) => quote::ToTokens::to_tokens(inner, tokens),
            AnyAttribute::Outer(outer) => quote::ToTokens::to_tokens(outer, tokens),
        }
    }
}

impl quote::ToTokens for RestrictedVis {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._pub, tokens);
        unsynn::ToTokens::to_tokens(&self.restriction, tokens);
    }
}

impl quote::ToTokens for ExternWithAbi {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        unsynn::ToTokens::to_tokens(&self._extern, tokens);
        unsynn::ToTokens::to_tokens(&self.abi, tokens);
    }
}

impl quote::ToTokens for ModuleItem {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            ModuleItem::TraitMethod(method) => quote::ToTokens::to_tokens(method, tokens),
            ModuleItem::Function(func) => quote::ToTokens::to_tokens(func, tokens),
            ModuleItem::ImplBlock(impl_block) => quote::ToTokens::to_tokens(impl_block, tokens),
            ModuleItem::Module(module) => quote::ToTokens::to_tokens(module, tokens),
            ModuleItem::Trait(trait_def) => quote::ToTokens::to_tokens(trait_def, tokens),
            ModuleItem::Enum(enum_sig) => quote::ToTokens::to_tokens(enum_sig, tokens),
            ModuleItem::Struct(struct_sig) => quote::ToTokens::to_tokens(struct_sig, tokens),
            ModuleItem::TypeAlias(type_alias) => quote::ToTokens::to_tokens(type_alias, tokens),
            ModuleItem::Const(const_sig) => quote::ToTokens::to_tokens(const_sig, tokens),
            ModuleItem::Static(static_sig) => quote::ToTokens::to_tokens(static_sig, tokens),
            ModuleItem::Other(token_tree) => unsynn::ToTokens::to_tokens(token_tree, tokens),
        }
    }
}

impl quote::ToTokens for ImplBlockSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }
        unsynn::ToTokens::to_tokens(&self._impl, tokens);
        if let Some(generics) = &self.generics {
            unsynn::ToTokens::to_tokens(generics, tokens);
        }
        unsynn::ToTokens::to_tokens(&self.target_type, tokens);
        if let Some(for_trait) = &self.for_trait {
            unsynn::ToTokens::to_tokens(for_trait, tokens);
        }
        if let Some(where_clause) = &self.where_clause {
            unsynn::ToTokens::to_tokens(where_clause, tokens);
        }
        unsynn::ToTokens::to_tokens(&self.items, tokens);
    }
}

impl quote::ToTokens for ModuleSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }
        if let Some(vis) = &self.visibility {
            quote::ToTokens::to_tokens(vis, tokens);
        }
        unsynn::ToTokens::to_tokens(&self._mod, tokens);
        quote::ToTokens::to_tokens(&self.name, tokens);
        unsynn::ToTokens::to_tokens(&self.items, tokens);
    }
}

impl quote::ToTokens for TraitSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }
        if let Some(vis) = &self.visibility {
            quote::ToTokens::to_tokens(vis, tokens);
        }
        if let Some(unsafe_kw) = &self.unsafe_kw {
            unsynn::ToTokens::to_tokens(unsafe_kw, tokens);
        }
        unsynn::ToTokens::to_tokens(&self._trait, tokens);
        quote::ToTokens::to_tokens(&self.name, tokens);
        if let Some(generics) = &self.generics {
            unsynn::ToTokens::to_tokens(generics, tokens);
        }
        if let Some(bounds) = &self.bounds {
            unsynn::ToTokens::to_tokens(bounds, tokens);
        }
        if let Some(where_clause) = &self.where_clause {
            unsynn::ToTokens::to_tokens(where_clause, tokens);
        }
        unsynn::ToTokens::to_tokens(&self.items, tokens);
    }
}

impl quote::ToTokens for ModuleContent {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // Output inner attributes first
        if let Some(inner_attrs) = &self.inner_attrs {
            for attr_delimited in &inner_attrs.0 {
                quote::ToTokens::to_tokens(&attr_delimited.value, tokens);
            }
        }
        unsynn::ToTokens::to_tokens(&self.items, tokens);
    }
}

impl quote::ToTokens for EnumSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }
        if let Some(vis) = &self.visibility {
            quote::ToTokens::to_tokens(vis, tokens);
        }
        unsynn::ToTokens::to_tokens(&self._enum, tokens);
        quote::ToTokens::to_tokens(&self.name, tokens);
        if let Some(generics) = &self.generics {
            unsynn::ToTokens::to_tokens(generics, tokens);
        }
        if let Some(where_clause) = &self.where_clause {
            unsynn::ToTokens::to_tokens(where_clause, tokens);
        }
        unsynn::ToTokens::to_tokens(&self.variants, tokens);
    }
}

impl quote::ToTokens for StructSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }
        if let Some(vis) = &self.visibility {
            quote::ToTokens::to_tokens(vis, tokens);
        }
        unsynn::ToTokens::to_tokens(&self._struct, tokens);
        quote::ToTokens::to_tokens(&self.name, tokens);
        if let Some(generics) = &self.generics {
            unsynn::ToTokens::to_tokens(generics, tokens);
        }
        if let Some(where_clause) = &self.where_clause {
            unsynn::ToTokens::to_tokens(where_clause, tokens);
        }
        quote::ToTokens::to_tokens(&self.body, tokens);
    }
}

impl quote::ToTokens for StructBody {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            StructBody::Named(fields) => unsynn::ToTokens::to_tokens(fields, tokens),
            StructBody::Tuple(tuple) => unsynn::ToTokens::to_tokens(tuple, tokens),
            StructBody::Unit(semi) => unsynn::ToTokens::to_tokens(semi, tokens),
        }
    }
}

impl quote::ToTokens for StructField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }
        if let Some(vis) = &self.visibility {
            quote::ToTokens::to_tokens(vis, tokens);
        }
        quote::ToTokens::to_tokens(&self.name, tokens);
        unsynn::ToTokens::to_tokens(&self._colon, tokens);
        unsynn::ToTokens::to_tokens(&self.field_type, tokens);
    }
}

impl quote::ToTokens for TypeAliasSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }
        if let Some(vis) = &self.visibility {
            quote::ToTokens::to_tokens(vis, tokens);
        }
        unsynn::ToTokens::to_tokens(&self._type, tokens);
        quote::ToTokens::to_tokens(&self.name, tokens);
        if let Some(generics) = &self.generics {
            unsynn::ToTokens::to_tokens(generics, tokens);
        }
        unsynn::ToTokens::to_tokens(&self._eq, tokens);
        unsynn::ToTokens::to_tokens(&self.target, tokens);
        unsynn::ToTokens::to_tokens(&self._semi, tokens);
    }
}

impl quote::ToTokens for ConstSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }
        if let Some(vis) = &self.visibility {
            quote::ToTokens::to_tokens(vis, tokens);
        }
        unsynn::ToTokens::to_tokens(&self._const, tokens);
        quote::ToTokens::to_tokens(&self.name, tokens);
        unsynn::ToTokens::to_tokens(&self._colon, tokens);
        unsynn::ToTokens::to_tokens(&self.const_type, tokens);
        unsynn::ToTokens::to_tokens(&self._eq, tokens);
        unsynn::ToTokens::to_tokens(&self.value, tokens);
        unsynn::ToTokens::to_tokens(&self._semi, tokens);
    }
}

impl quote::ToTokens for StaticSig {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }
        if let Some(vis) = &self.visibility {
            quote::ToTokens::to_tokens(vis, tokens);
        }
        if let Some(mut_kw) = &self.mut_kw {
            unsynn::ToTokens::to_tokens(mut_kw, tokens);
        }
        unsynn::ToTokens::to_tokens(&self._static, tokens);
        quote::ToTokens::to_tokens(&self.name, tokens);
        unsynn::ToTokens::to_tokens(&self._colon, tokens);
        unsynn::ToTokens::to_tokens(&self.static_type, tokens);
        unsynn::ToTokens::to_tokens(&self._eq, tokens);
        unsynn::ToTokens::to_tokens(&self.value, tokens);
        unsynn::ToTokens::to_tokens(&self._semi, tokens);
    }
}

impl quote::ToTokens for EnumVariant {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(attrs) = &self.attributes {
            for attr in &attrs.0 {
                unsynn::ToTokens::to_tokens(attr, tokens);
            }
        }
        quote::ToTokens::to_tokens(&self.name, tokens);
        if let Some(data) = &self.data {
            quote::ToTokens::to_tokens(data, tokens);
        }
    }
}

impl quote::ToTokens for EnumVariantData {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            EnumVariantData::Tuple(paren) => unsynn::ToTokens::to_tokens(paren, tokens),
            EnumVariantData::Struct(brace) => unsynn::ToTokens::to_tokens(brace, tokens),
            EnumVariantData::Discriminant(disc) => unsynn::ToTokens::to_tokens(disc, tokens),
        }
    }
}

#[cfg(test)]
mod tests;
