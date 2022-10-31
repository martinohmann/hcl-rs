use super::Expression;
use crate::Identifier;
use serde::{Deserialize, Serialize};

/// A for expression is a construct for constructing a collection by projecting the items from
/// another collection.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename = "$hcl::for_expr")]
pub struct ForExpr {
    /// Optional name of the variable that will be temporarily assigned the key of each element
    /// during iteration. If the source collection is an array, it gets assigned the zero-based
    /// array index. For an object source collection, this gets assigned the object's key.
    pub key_var: Option<Identifier>,
    /// The name of the variable that will be temporarily assigned the value of each element
    /// during iteration.
    pub value_var: Identifier,
    /// An expression that must evaluate to a value that can be iterated.
    pub collection_expr: Expression,
    /// An expression that is evaluated once for each key in the source collection. If set, the
    /// result of the `for` expression will be an object. Otherwise, the result will be an array.
    pub key_expr: Option<Expression>,
    /// An expression that is evaluated once for each value in the source collection.
    pub value_expr: Expression,
    /// Indicates whether grouping mode is enabled. In grouping mode, each value in the resulting
    /// object is a list of all of the values that were produced against each distinct key. This is
    /// ignored if `key_expr` is `None`.
    pub grouping: bool,
    /// An optional filter expression. Elements for which the condition evaluates to `true` will
    /// be evaluated as normal, while if `false` the element will be skipped.
    pub cond_expr: Option<Expression>,
}

impl ForExpr {
    /// Create a new `ForExpr` with the name of the variable that will be temporarily assigned the
    /// value of each element during iteration, an expression that must evaluate to a value that
    /// can be iterated, and one expression that is evaluated once for each value in the source
    /// collection.
    pub fn new<C, V>(value_var: Identifier, collection_expr: C, value_expr: V) -> ForExpr
    where
        C: Into<Expression>,
        V: Into<Expression>,
    {
        ForExpr {
            key_var: None,
            value_var,
            collection_expr: collection_expr.into(),
            key_expr: None,
            value_expr: value_expr.into(),
            grouping: false,
            cond_expr: None,
        }
    }

    /// Adds the iterator key variable identifier to the `for` expression and returns the modified
    /// `ForExpr`.
    pub fn with_key_var(mut self, key_var: Identifier) -> ForExpr {
        self.key_var = Some(key_var);
        self
    }

    /// Adds an expression that is evaluated once for each key in the source collection. If set,
    /// the result of the `for` expression will be an object. Returns the modified `ForExpr`.
    pub fn with_key_expr<T>(mut self, key_expr: T) -> ForExpr
    where
        T: Into<Expression>,
    {
        self.key_expr = Some(key_expr.into());
        self
    }

    /// Sets the filter expression. Elements for which the condition evaluates to `true` will be
    /// evaluated as normal, while if `false` the element will be skipped.
    pub fn with_cond_expr<T>(mut self, cond_expr: T) -> ForExpr
    where
        T: Into<Expression>,
    {
        self.cond_expr = Some(cond_expr.into());
        self
    }

    /// Enables or disabled grouping mode.
    pub fn with_grouping(mut self, yes: bool) -> ForExpr {
        self.grouping = yes;
        self
    }
}
