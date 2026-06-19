extern crate self as inquiry_sqlx;

pub use inquiry_sqlx_macros::Queryable;

/// Operators for equality comparisons and text pattern matching.
#[derive(Clone, Copy, Debug)]
pub enum QueryOperator {
    /// Matches rows where the field equals the given value.
    Eq,
    /// Matches rows where the field does not equal the given value.
    Ne,
    /// Matches text rows using a case-sensitive SQL `LIKE` pattern.
    Like,
    /// Matches text rows using a case-insensitive SQL `ILIKE` pattern.
    ILike,
}

impl QueryOperator {
    #[doc(hidden)]
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "!=",
            Self::Like => "LIKE",
            Self::ILike => "ILIKE",
        }
    }
}

/// Operators for fields that only support equality comparisons.
#[derive(Clone, Copy, Debug)]
pub enum QueryEqualityOperator {
    /// Matches rows where the field equals the given value.
    Eq,
    /// Matches rows where the field does not equal the given value.
    Ne,
}

impl QueryEqualityOperator {
    #[doc(hidden)]
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "!=",
        }
    }
}

/// Ordering operators for fields that support range comparisons.
#[derive(Clone, Copy, Debug)]
pub enum QueryOrderingOperator {
    /// Matches rows where the field equals the given value.
    Eq,
    /// Matches rows where the field does not equal the given value.
    Ne,
    /// Matches rows where the field is greater than the given value.
    Gt,
    /// Matches rows where the field is greater than or equal to the given value.
    Gte,
    /// Matches rows where the field is less than the given value.
    Lt,
    /// Matches rows where the field is less than or equal to the given value.
    Lte,
}

impl QueryOrderingOperator {
    #[doc(hidden)]
    pub fn as_sql(self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Ne => "!=",
            Self::Gt => ">",
            Self::Gte => ">=",
            Self::Lt => "<",
            Self::Lte => "<=",
        }
    }
}
