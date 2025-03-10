use crate::errors::{NotFixedSize, TypeError};
use crate::namespace::items::{Class, ContractId, StructId};
use crate::AnalyzerDb;

use num_bigint::BigInt;
use num_traits::ToPrimitive;
use smol_str::SmolStr;
use std::fmt;
use std::str::FromStr;
use strum::{AsRefStr, EnumIter, EnumString};
use vec1::Vec1;

pub fn u256_min() -> BigInt {
    BigInt::from(0)
}

pub fn u256_max() -> BigInt {
    BigInt::from(2).pow(256) - 1
}

pub fn i256_max() -> BigInt {
    BigInt::from(2).pow(255) - 1
}

pub fn i256_min() -> BigInt {
    BigInt::from(-2).pow(255)
}

/// Names that can be used to build identifiers without collision.
pub trait SafeNames {
    /// Name in the lower snake format (e.g. lower_snake_case).
    fn lower_snake(&self) -> String;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Base(Base),
    Array(Array),
    Map(Map),
    Tuple(Tuple),
    String(FeString),
    /// An "external" contract. Effectively just a `newtype`d address.
    Contract(Contract),
    /// The type of a contract while it's being executed. Ie. the type
    /// of `self` within a contract function.
    SelfContract(Contract),
    Struct(Struct),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum FixedSize {
    Base(Base),
    Array(Array),
    Tuple(Tuple),
    String(FeString),
    Contract(Contract),
    Struct(Struct),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Base {
    Numeric(Integer),
    Bool,
    Address,
    Unit,
}

impl Base {
    pub fn name(&self) -> SmolStr {
        match self {
            Base::Numeric(num) => num.as_ref().into(),
            Base::Bool => "bool".into(),
            Base::Address => "address".into(),
            Base::Unit => "()".into(),
        }
    }
    pub fn u256() -> Base {
        Base::Numeric(Integer::U256)
    }
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, AsRefStr, EnumString, EnumIter,
)]
#[strum(serialize_all = "snake_case")]
pub enum Integer {
    U256,
    U128,
    U64,
    U32,
    U16,
    U8,
    I256,
    I128,
    I64,
    I32,
    I16,
    I8,
}

pub const U256: Base = Base::Numeric(Integer::U256);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Array {
    pub size: usize,
    pub inner: Base,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Map {
    pub key: Base,
    pub value: Box<Type>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tuple {
    pub items: Vec1<FixedSize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Struct {
    pub name: SmolStr,
    pub id: StructId,
    pub field_count: usize,
}
impl Struct {
    pub fn from_id(id: StructId, db: &dyn AnalyzerDb) -> Self {
        Self {
            name: id.name(db),
            id,
            field_count: id.fields(db).len(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Contract {
    pub name: SmolStr,
    pub id: ContractId,
}
impl Contract {
    pub fn from_id(id: ContractId, db: &dyn AnalyzerDb) -> Self {
        Self {
            name: id.name(db),
            id,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct FeString {
    pub max_size: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionSignature {
    pub self_decl: Option<SelfDecl>,
    pub params: Vec<FunctionParam>,
    pub return_type: Result<FixedSize, TypeError>,
}

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub enum SelfDecl {
    Mutable,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FunctionParam {
    pub name: SmolStr,
    pub typ: Result<FixedSize, TypeError>,
}

#[derive(
    Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, EnumString, AsRefStr, EnumIter,
)]
pub enum GenericType {
    Array,
    String,
    Map,
}

impl GenericType {
    pub fn name(&self) -> SmolStr {
        self.as_ref().into()
    }
    pub fn params(&self) -> Vec<GenericParam> {
        match self {
            GenericType::String => vec![GenericParam {
                name: "max size".into(),
                kind: GenericParamKind::Int,
            }],
            GenericType::Map => vec![
                GenericParam {
                    name: "key".into(),
                    kind: GenericParamKind::PrimitiveType,
                },
                GenericParam {
                    name: "value".into(),
                    kind: GenericParamKind::AnyType,
                },
            ],
            GenericType::Array => vec![
                GenericParam {
                    name: "element type".into(),
                    kind: GenericParamKind::PrimitiveType,
                },
                GenericParam {
                    name: "size".into(),
                    kind: GenericParamKind::Int,
                },
            ],
        }
    }

    // see traversal::types::apply_generic_type_args for error checking
    pub fn apply(&self, args: &[GenericArg]) -> Option<Type> {
        match self {
            GenericType::String => match args {
                [GenericArg::Int(max_size)] => Some(Type::String(FeString {
                    max_size: *max_size,
                })),
                _ => None,
            },
            GenericType::Map => match args {
                [GenericArg::Type(key), GenericArg::Type(value)] => Some(Type::Map(Map {
                    key: key.as_primitive()?,
                    value: Box::new(value.clone()),
                })),
                _ => None,
            },
            GenericType::Array => match args {
                [GenericArg::Type(element), GenericArg::Int(size)] => Some(Type::Array(Array {
                    size: *size,
                    inner: element.as_primitive()?,
                })),
                _ => None,
            },
        }
    }
}

pub struct GenericParam {
    pub name: SmolStr,
    pub kind: GenericParamKind,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenericParamKind {
    Int,

    // Ideally these would be represented as trait constraints.
    PrimitiveType,
    // FixedSizeType, // not needed yet
    AnyType,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GenericArg {
    Int(usize),
    Type(Type),
}

impl Integer {
    /// Returns `true` if the integer is signed, otherwise `false`
    pub fn is_signed(&self) -> bool {
        matches!(
            self,
            Integer::I256
                | Integer::I128
                | Integer::I64
                | Integer::I32
                | Integer::I16
                | Integer::I8
        )
    }

    pub fn size(&self) -> usize {
        match self {
            Integer::U256 => 32,
            Integer::U128 => 16,
            Integer::U64 => 8,
            Integer::U32 => 4,
            Integer::U16 => 2,
            Integer::U8 => 1,
            Integer::I256 => 32,
            Integer::I128 => 16,
            Integer::I64 => 8,
            Integer::I32 => 4,
            Integer::I16 => 2,
            Integer::I8 => 1,
        }
    }

    /// Returns size of integer type in bits.
    pub fn bits(&self) -> usize {
        self.size() * 8
    }

    /// Returns `true` if the integer is at least the same size (or larger) than
    /// `other`
    pub fn can_hold(&self, other: &Integer) -> bool {
        self.size() >= other.size()
    }

    /// Returns `true` if `num` represents a number that fits the type
    pub fn fits(&self, num: BigInt) -> bool {
        match self {
            Integer::U8 => num.to_u8().is_some(),
            Integer::U16 => num.to_u16().is_some(),
            Integer::U32 => num.to_u32().is_some(),
            Integer::U64 => num.to_u64().is_some(),
            Integer::U128 => num.to_u128().is_some(),
            Integer::I8 => num.to_i8().is_some(),
            Integer::I16 => num.to_i16().is_some(),
            Integer::I32 => num.to_i32().is_some(),
            Integer::I64 => num.to_i64().is_some(),
            Integer::I128 => num.to_i128().is_some(),
            Integer::U256 => num >= u256_min() && num <= u256_max(),
            Integer::I256 => num >= i256_min() && num <= i256_max(),
        }
    }

    /// Returns max value of the integer type.
    pub fn max_value(&self) -> BigInt {
        match self {
            Integer::U256 => u256_max(),
            Integer::U128 => u128::MAX.into(),
            Integer::U64 => u64::MAX.into(),
            Integer::U32 => u32::MAX.into(),
            Integer::U16 => u16::MAX.into(),
            Integer::U8 => u8::MAX.into(),
            Integer::I256 => i256_max(),
            Integer::I128 => i128::MAX.into(),
            Integer::I64 => i64::MAX.into(),
            Integer::I32 => i32::MAX.into(),
            Integer::I16 => i16::MAX.into(),
            Integer::I8 => i8::MAX.into(),
        }
    }

    /// Returns min value of the integer type.
    pub fn min_value(&self) -> BigInt {
        match self {
            Integer::U256 => u256_min(),
            Integer::U128 => u128::MIN.into(),
            Integer::U64 => u64::MIN.into(),
            Integer::U32 => u32::MIN.into(),
            Integer::U16 => u16::MIN.into(),
            Integer::U8 => u8::MIN.into(),
            Integer::I256 => i256_min(),
            Integer::I128 => i128::MIN.into(),
            Integer::I64 => i64::MIN.into(),
            Integer::I32 => i32::MIN.into(),
            Integer::I16 => i16::MIN.into(),
            Integer::I8 => i8::MIN.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Event {
    pub name: SmolStr,
    pub fields: Vec<EventField>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EventField {
    pub name: SmolStr,
    pub typ: Result<FixedSize, TypeError>,
    pub is_indexed: bool,
}

impl FunctionSignature {
    /// # Panics
    /// Panics if any param type is an `Err`
    pub fn param_types(&self) -> Vec<FixedSize> {
        self.params
            .iter()
            .map(|param| param.typ.clone().expect("fn param type error"))
            .collect()
    }

    /// # Panics
    /// Panics if the return type is an `Err`
    pub fn expect_return_type(&self) -> FixedSize {
        self.return_type.clone().expect("fn return type error")
    }
}

impl Type {
    pub fn name(&self) -> SmolStr {
        match self {
            Type::Base(inner) => inner.name(),
            Type::Array(inner) => inner.to_string().into(),
            Type::Map(inner) => inner.to_string().into(),
            Type::Tuple(inner) => inner.to_string().into(),
            Type::String(inner) => inner.to_string().into(),
            Type::Contract(inner) => inner.name.clone(),
            Type::SelfContract(inner) => inner.name.clone(),
            Type::Struct(inner) => inner.name.clone(),
        }
    }

    /// Returns `true` if the type is integer.
    pub fn is_integer(&self) -> bool {
        matches!(self, Type::Base(Base::Numeric(_)))
    }

    pub fn is_signed_integer(&self) -> bool {
        if let Type::Base(Base::Numeric(integer)) = &self {
            return integer.is_signed();
        }
        false
    }

    pub fn unit() -> Self {
        Type::Base(Base::Unit)
    }
    pub fn is_unit(&self) -> bool {
        *self == Type::Base(Base::Unit)
    }

    pub fn int(int_type: Integer) -> Self {
        Type::Base(Base::Numeric(int_type))
    }

    pub fn generic_arg_type(&self, idx: usize) -> Option<Type> {
        match self {
            Type::Map(Map { key, value }) => match idx {
                0 => Some(Type::Base(*key)),
                1 => Some((**value).clone()),
                _ => None,
            },
            Type::Array(array) => match idx {
                0 => Some(Type::Base(array.inner)),
                _ => None,
            },
            _ => None,
        }
    }
}

pub trait TypeDowncast {
    fn as_array(&self) -> Option<&Array>;
    fn as_tuple(&self) -> Option<&Tuple>;
    fn as_string(&self) -> Option<&FeString>;
    fn as_map(&self) -> Option<&Map>;
    fn as_int(&self) -> Option<Integer>;
    fn as_primitive(&self) -> Option<Base>;
    fn as_class(&self) -> Option<Class>;
}

impl TypeDowncast for Type {
    fn as_array(&self) -> Option<&Array> {
        match self {
            Type::Array(inner) => Some(inner),
            _ => None,
        }
    }
    fn as_tuple(&self) -> Option<&Tuple> {
        match self {
            Type::Tuple(inner) => Some(inner),
            _ => None,
        }
    }
    fn as_string(&self) -> Option<&FeString> {
        match self {
            Type::String(inner) => Some(inner),
            _ => None,
        }
    }
    fn as_map(&self) -> Option<&Map> {
        match self {
            Type::Map(inner) => Some(inner),
            _ => None,
        }
    }
    fn as_int(&self) -> Option<Integer> {
        match self {
            Type::Base(Base::Numeric(int)) => Some(*int),
            _ => None,
        }
    }
    fn as_primitive(&self) -> Option<Base> {
        match self {
            Type::Base(base) => Some(*base),
            _ => None,
        }
    }
    fn as_class(&self) -> Option<Class> {
        match self {
            Type::Struct(inner) => Some(Class::Struct(inner.id)),
            Type::Contract(inner) => Some(Class::Contract(inner.id)),
            Type::SelfContract(inner) => Some(Class::Contract(inner.id)),
            _ => None,
        }
    }
}

impl TypeDowncast for Option<&Type> {
    fn as_array(&self) -> Option<&Array> {
        self.and_then(|t| t.as_array())
    }
    fn as_tuple(&self) -> Option<&Tuple> {
        self.and_then(|t| t.as_tuple())
    }
    fn as_string(&self) -> Option<&FeString> {
        self.and_then(|t| t.as_string())
    }
    fn as_map(&self) -> Option<&Map> {
        self.and_then(|t| t.as_map())
    }
    fn as_int(&self) -> Option<Integer> {
        self.and_then(|t| t.as_int())
    }
    fn as_primitive(&self) -> Option<Base> {
        self.and_then(|t| t.as_primitive())
    }
    fn as_class(&self) -> Option<Class> {
        self.and_then(|t| t.as_class())
    }
}

impl From<FixedSize> for Type {
    fn from(value: FixedSize) -> Self {
        match value {
            FixedSize::Array(array) => Type::Array(array),
            FixedSize::Base(base) => Type::Base(base),
            FixedSize::Tuple(tuple) => Type::Tuple(tuple),
            FixedSize::String(string) => Type::String(string),
            FixedSize::Contract(contract) => Type::Contract(contract),
            FixedSize::Struct(val) => Type::Struct(val),
        }
    }
}

impl From<Base> for Type {
    fn from(value: Base) -> Self {
        Type::Base(value)
    }
}

impl FixedSize {
    /// Returns true if the type is `()`.
    pub fn is_unit(&self) -> bool {
        self == &Self::Base(Base::Unit)
    }

    pub fn is_base(&self) -> bool {
        matches!(self, FixedSize::Base(_))
    }

    /// Creates an instance of bool.
    pub fn bool() -> Self {
        FixedSize::Base(Base::Bool)
    }

    /// Creates an instance of address.
    pub fn address() -> Self {
        FixedSize::Base(Base::Address)
    }

    /// Creates an instance of u256.
    pub fn u256() -> Self {
        FixedSize::Base(Base::Numeric(Integer::U256))
    }

    /// Creates an instance of u8.
    pub fn u8() -> Self {
        FixedSize::Base(Base::Numeric(Integer::U8))
    }

    /// Creates an instance of `()`.
    pub fn unit() -> Self {
        FixedSize::Base(Base::Unit)
    }
}

impl PartialEq<Type> for FixedSize {
    fn eq(&self, other: &Type) -> bool {
        match (self, other) {
            (FixedSize::Array(in1), Type::Array(in2)) => in1 == in2,
            (FixedSize::Base(in1), Type::Base(in2)) => in1 == in2,
            (FixedSize::Tuple(in1), Type::Tuple(in2)) => in1 == in2,
            (FixedSize::String(in1), Type::String(in2)) => in1 == in2,
            (FixedSize::Contract(in1), Type::Contract(in2)) => in1 == in2,
            (FixedSize::Struct(in1), Type::Struct(in2)) => in1 == in2,
            _ => false,
        }
    }
}

impl From<Base> for FixedSize {
    fn from(value: Base) -> Self {
        FixedSize::Base(value)
    }
}

impl From<FeString> for FixedSize {
    fn from(value: FeString) -> Self {
        FixedSize::String(value)
    }
}

impl TryFrom<Type> for FixedSize {
    type Error = NotFixedSize;

    fn try_from(value: Type) -> Result<Self, NotFixedSize> {
        match value {
            Type::Array(array) => Ok(FixedSize::Array(array)),
            Type::Base(base) => Ok(FixedSize::Base(base)),
            Type::Tuple(tuple) => Ok(FixedSize::Tuple(tuple)),
            Type::String(string) => Ok(FixedSize::String(string)),
            Type::Struct(val) => Ok(FixedSize::Struct(val)),
            Type::Map(_) => Err(NotFixedSize),
            Type::Contract(contract) => Ok(FixedSize::Contract(contract)),
            Type::SelfContract(_) => Err(NotFixedSize),
        }
    }
}

impl From<Tuple> for FixedSize {
    fn from(value: Tuple) -> Self {
        FixedSize::Tuple(value)
    }
}

impl SafeNames for FixedSize {
    fn lower_snake(&self) -> String {
        match self {
            FixedSize::Array(array) => array.lower_snake(),
            FixedSize::Base(base) => base.lower_snake(),
            FixedSize::Tuple(tuple) => tuple.lower_snake(),
            FixedSize::String(string) => string.lower_snake(),
            FixedSize::Contract(contract) => contract.lower_snake(),
            FixedSize::Struct(val) => val.lower_snake(),
        }
    }
}

impl SafeNames for Base {
    fn lower_snake(&self) -> String {
        match self {
            Base::Numeric(Integer::U256) => "u256".to_string(),
            Base::Numeric(Integer::U128) => "u128".to_string(),
            Base::Numeric(Integer::U64) => "u64".to_string(),
            Base::Numeric(Integer::U32) => "u32".to_string(),
            Base::Numeric(Integer::U16) => "u16".to_string(),
            Base::Numeric(Integer::U8) => "u8".to_string(),
            Base::Numeric(Integer::I256) => "i256".to_string(),
            Base::Numeric(Integer::I128) => "i128".to_string(),
            Base::Numeric(Integer::I64) => "i64".to_string(),
            Base::Numeric(Integer::I32) => "i32".to_string(),
            Base::Numeric(Integer::I16) => "i16".to_string(),
            Base::Numeric(Integer::I8) => "i8".to_string(),
            Base::Address => "address".to_string(),
            Base::Bool => "bool".to_string(),
            Base::Unit => "unit".to_string(),
        }
    }
}

impl SafeNames for Array {
    fn lower_snake(&self) -> String {
        format!("array_{}_{}", self.inner.lower_snake(), self.size)
    }
}

impl SafeNames for Struct {
    fn lower_snake(&self) -> String {
        format!("struct_{}", self.name)
    }
}

impl SafeNames for Tuple {
    fn lower_snake(&self) -> String {
        let field_names = self
            .items
            .iter()
            .map(|typ| typ.lower_snake())
            .collect::<Vec<String>>();
        let joined_names = field_names.join("_");

        // The trailing `_` denotes the end of the tuple, to differentiate between
        // different tuple nestings. Eg
        // (A, (B, C), D) => tuple_A_tuple_B_C__D_
        // (A, (B, C, D)) => tuple_A_tuple_B_C_D__
        // Conceptually, each paren and comma is replaced with an underscore.
        format!("tuple_{}_", joined_names)
    }
}

impl SafeNames for Contract {
    fn lower_snake(&self) -> String {
        unimplemented!();
    }
}

impl SafeNames for FeString {
    fn lower_snake(&self) -> String {
        format!("string_{}", self.max_size)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Base(inner) => inner.fmt(f),
            Type::Array(inner) => inner.fmt(f),
            Type::Map(inner) => inner.fmt(f),
            Type::Tuple(inner) => inner.fmt(f),
            Type::String(inner) => inner.fmt(f),
            Type::Contract(inner) => inner.fmt(f),
            Type::SelfContract(inner) => inner.fmt(f),
            Type::Struct(inner) => inner.fmt(f),
        }
    }
}

impl fmt::Display for FixedSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FixedSize::Base(inner) => inner.fmt(f),
            FixedSize::Array(inner) => inner.fmt(f),
            FixedSize::Tuple(inner) => inner.fmt(f),
            FixedSize::String(inner) => inner.fmt(f),
            FixedSize::Contract(inner) => inner.fmt(f),
            FixedSize::Struct(inner) => inner.fmt(f),
        }
    }
}

impl fmt::Display for Base {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Base::Numeric(int) => return int.fmt(f),
            Base::Bool => "bool",
            Base::Address => "address",
            Base::Unit => "()",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for Integer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Integer::U256 => "u256",
            Integer::U128 => "u128",
            Integer::U64 => "u64",
            Integer::U32 => "u32",
            Integer::U16 => "u16",
            Integer::U8 => "u8",
            Integer::I256 => "i256",
            Integer::I128 => "i128",
            Integer::I64 => "i64",
            Integer::I32 => "i32",
            Integer::I16 => "i16",
            Integer::I8 => "i8",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for Array {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Array<{}, {}>", self.inner, self.size)
    }
}

impl fmt::Display for Map {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Map<{}, {}>", self.key, self.value)
    }
}

impl fmt::Display for Tuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(")?;
        let mut delim = "";
        for item in &self.items {
            write!(f, "{}{}", delim, item)?;
            delim = ", ";
        }
        write!(f, ")")
    }
}

impl fmt::Display for FeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "String<{}>", self.max_size)
    }
}

impl fmt::Display for Contract {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Display for Struct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl FromStr for Base {
    type Err = strum::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bool" => Ok(Base::Bool),
            "address" => Ok(Base::Address),
            "()" => Ok(Base::Unit),
            _ => Ok(Base::Numeric(Integer::from_str(s)?)),
        }
    }
}
