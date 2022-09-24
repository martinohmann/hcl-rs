use super::{Expression, Identifier};
use serde::{Deserialize, Serialize};

/// A for expression is a construct for constructing a collection by projecting the items from
/// another collection.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename = "$hcl::for_expr")]
pub enum ForExpr {
    /// Represents a `for` expression that produces a list.
    List(ForListExpr),
    /// Represents a `for` expression that produces an object.
    Object(ForObjectExpr),
}

impl From<ForListExpr> for ForExpr {
    fn from(expr: ForListExpr) -> Self {
        ForExpr::List(expr)
    }
}

impl From<ForObjectExpr> for ForExpr {
    fn from(expr: ForObjectExpr) -> Self {
        ForExpr::Object(expr)
    }
}

/// A `for` expression that produces a list.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename = "$hcl::for_list_expr")]
pub struct ForListExpr {
    /// Optional name of the variable that will be temporarily assigned the zero-based index of
    /// each element during iteration.
    pub index_var: Option<Identifier>,
    /// The name of the variable that will be temporarily assigned the value of each element
    /// during iteration.
    pub value_var: Identifier,
    /// An expression that must evaluate to an array value that can be iterated.
    pub collection_expr: Expression,
    /// An expression that is evaluated once for each element in the source collection.
    pub element_expr: Expression,
    /// An optional filter expression. Elements for which the condition evaluates to `true` will
    /// be evaluated as normal, while if `false` the element will be skipped.
    pub cond_expr: Option<Expression>,
}

impl ForListExpr {
    /// Create a new `ForListExpr` with the name of the variable that will be temporarily assigned
    /// the value of each element during iteration, an expression that must evaluate to an array
    /// and an expression that is evaluated once for each element in the source array.
    pub fn new<C, E>(value_var: Identifier, collection_expr: C, element_expr: E) -> ForListExpr
    where
        C: Into<Expression>,
        E: Into<Expression>,
    {
        ForListExpr {
            index_var: None,
            value_var,
            collection_expr: collection_expr.into(),
            element_expr: element_expr.into(),
            cond_expr: None,
        }
    }

    /// Adds the iterator index variable identifier to the `for` expression and returns the
    /// modified `ForListExpr`.
    pub fn with_index_var(mut self, index_var: Identifier) -> ForListExpr {
        self.index_var = Some(index_var);
        self
    }

    /// Sets the filter expression. Elements for which the condition evaluates to `true` will be
    /// evaluated as normal, while if `false` the element will be skipped.
    pub fn with_cond_expr<T>(mut self, cond_expr: T) -> ForListExpr
    where
        T: Into<Expression>,
    {
        self.cond_expr = Some(cond_expr.into());
        self
    }
}

/// A `for` expression that produces an object.
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename = "$hcl::for_object_expr")]
pub struct ForObjectExpr {
    /// Optional name of the variable that will be temporarily assigned the object key of each
    /// element during iteration.
    pub key_var: Option<Identifier>,
    /// The name of the variable that will be temporarily assigned the value of each element
    /// during iteration.
    pub value_var: Identifier,
    /// An expression that must evaluate to an object value that can be iterated.
    pub collection_expr: Expression,
    /// An expression that is evaluated once for each element key in the source collection.
    pub key_expr: Expression,
    /// An expression that is evaluated once for each element value in the source collection.
    pub value_expr: Expression,
    /// Indicates whether grouping mode is enabled. In grouping mode, each value in the resulting
    /// object is a list of all of the values that were produced against each distinct key.
    pub grouping: bool,
    /// An optional filter expression. Elements for which the condition evaluates to `true` will
    /// be evaluated as normal, while if `false` the element will be skipped.
    pub cond_expr: Option<Expression>,
}

impl ForObjectExpr {
    /// Create a new `ForObjectExpr` with the name of the variable that will be temporarily
    /// assigned the value of each element during iteration, an expression that must evaluate to
    /// an object, and two expressions that are evaluated once for each element's key and value in
    /// the source object.
    pub fn new<C, K, V>(
        value_var: Identifier,
        collection_expr: C,
        key_expr: K,
        value_expr: V,
    ) -> ForObjectExpr
    where
        C: Into<Expression>,
        K: Into<Expression>,
        V: Into<Expression>,
    {
        ForObjectExpr {
            key_var: None,
            value_var,
            collection_expr: collection_expr.into(),
            key_expr: key_expr.into(),
            value_expr: value_expr.into(),
            grouping: false,
            cond_expr: None,
        }
    }

    /// Adds the iterator key variable identifier to the `for` expression and returns the modified
    /// `ForObjectExpr`.
    pub fn with_key_var(mut self, key_var: Identifier) -> ForObjectExpr {
        self.key_var = Some(key_var);
        self
    }

    /// Sets the filter expression. Elements for which the condition evaluates to `true` will be
    /// evaluated as normal, while if `false` the element will be skipped.
    pub fn with_cond_expr<T>(mut self, cond_expr: T) -> ForObjectExpr
    where
        T: Into<Expression>,
    {
        self.cond_expr = Some(cond_expr.into());
        self
    }

    /// Enables or disabled grouping mode.
    pub fn with_grouping(mut self, yes: bool) -> ForObjectExpr {
        self.grouping = yes;
        self
    }
}
