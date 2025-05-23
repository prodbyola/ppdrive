use crate::{errors::AppError, utils::sqlx::filter::filter_parser};
use std::fmt::Display;

use super::filter::filter_to_query;

/// Generates compatible SQL string for defined Sqlx types
pub trait ToQuery {
    fn to_query(&self, bn: &BackendName) -> String;
}

#[derive(Clone)]
/// Rust representations for [sqlx::PoolConnection<Any>::backend_name].
pub enum BackendName {
    Postgres,
    Mysql,
    Sqlite,
}

impl BackendName {
    pub fn to_query(&self, index: u8) -> String {
        use BackendName::*;

        match self {
            Postgres => format!("${}", index),
            Sqlite => format!("?{}", index),
            Mysql => "?".to_string(),
        }
    }
}

impl<'a> TryFrom<&'a str> for BackendName {
    type Error = AppError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if value == "PostgreSQL" {
            Ok(Self::Postgres)
        } else if value == "MySQL" {
            Ok(Self::Mysql)
        } else if value == "SQLite" {
            Ok(Self::Sqlite)
        } else {
            Err(AppError::DatabaseError(format!(
                "unable to parse backend name {value}"
            )))
        }
    }
}

/// For generating compatible filter (WHERE) SQL string
pub(super) enum Filter<'a> {
    Base(&'a str),
    And(&'a str),
    Or(&'a str),
    Group(Vec<Filter<'a>>),
    AndGroup(Box<Filter<'a>>),
    OrGroup(Box<Filter<'a>>),
}

impl Display for Filter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Filter::*;

        match self {
            Base(col) => write!(f, "{col}"),
            And(col) => write!(f, "AND {col}"),
            Or(col) => write!(f, "OR {col}"),
            _ => write!(f, ""),
        }
    }
}

impl<'a> TryFrom<&'a str> for Filter<'a> {
    type Error = AppError;
    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        filter_parser(value)
    }
}

/// A wrapper that holds a chain of [Filter]s which can be converted
/// into sql query string.
///
///
/// Example:
/// ```
/// let filters = SqlxFilters::("id");
/// filters.and("age");
/// filters.to_query(&bn); // id = $1 AND age = $1
/// ```
pub struct SqlxFilters<'a> {
    items: Vec<&'a str>,
    offset: u8,
}

impl<'a> SqlxFilters<'a> {
    pub fn new(filter: &'a str, offset: u8) -> Self {
        SqlxFilters {
            items: Vec::from([filter]),
            offset,
        }
    }

    pub fn add(mut self, filter: &'a str) -> Self {
        self.items.push(filter);
        self
    }

    pub fn to_query(&self, bn: &BackendName) -> Result<String, AppError> {
        filter_to_query(&self.items, &self.offset, bn)
    }
}

/// Generates query string with compatible placeholders for SQL VALUES. Allows you to
/// provide how many placeholders you would like to generate.
///
///
/// Example:
/// ```
/// let values = SqlxValues(3);
/// values.to_query(bn)?; // VALUES($1, $2, $3)
/// ```
pub struct SqlxValues(pub u8, pub u8);
impl ToQuery for SqlxValues {
    fn to_query(&self, bn: &BackendName) -> String {
        let mut values = Vec::with_capacity(self.0 as usize);

        for i in 0..self.0 {
            let bq = bn.to_query(i + self.1);
            values.push(bq);
        }

        let values = values.join(", ");
        format!("VALUES({values})")
    }
}

pub struct SqlxSetters<'a> {
    items: Vec<&'a str>,
    offset: u8,
}

impl<'a> SqlxSetters<'a> {
    pub fn new(col: &'a str, offset: u8) -> Self {
        Self {
            items: vec![col],
            offset,
        }
    }

    pub fn add(mut self, col: &'a str) -> Self {
        self.items.push(col);
        self
    }
}

impl ToQuery for SqlxSetters<'_> {
    fn to_query(&self, bn: &BackendName) -> String {
        let out: Vec<String> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, col)| {
                let bq = bn.to_query(i as u8 + self.offset);
                format!("{col} = {bq}")
            })
            .collect();

        out.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::AppError;

    use super::{BackendName, SqlxFilters, SqlxValues, ToQuery};

    #[test]
    fn test_sqlx_filters_pg() -> Result<(), AppError> {
        // single condition
        let filters = SqlxFilters::new("id", 1);
        let bn = BackendName::Postgres;

        assert_eq!(&filters.to_query(&bn)?, "id = $1");

        // multiple conditions
        let filters = filters.add("AND age").add("OR name");
        assert_eq!(&filters.to_query(&bn)?, "id = $1 AND age = $2 OR name = $3");

        // grouped conditions (prefix)
        let filters = SqlxFilters::new("asset_path OR custom_path", 1)
            .add("AND asset_type")
            .to_query(&bn)?;

        assert_eq!(
            &filters,
            "(asset_path = $1 OR custom_path = $2) AND asset_type = $3"
        );

        // grouped conditions (suffix)
        let filters = SqlxFilters::new("asset_type", 1)
            .add("AND (asset_path OR custom_path)")
            .to_query(&bn)?;

        assert_eq!(
            &filters,
            "asset_type = $1 AND (asset_path = $2 OR custom_path = $3)"
        );
        Ok(())
    }

    #[test]
    fn test_sqlx_values_pg() {
        let values = SqlxValues(1, 1);
        let bn = BackendName::Postgres;

        assert_eq!(&values.to_query(&bn), "VALUES($1)");

        let values = SqlxValues(3, 1);
        assert_eq!(&values.to_query(&bn), "VALUES($1, $2, $3)");
    }

    #[test]
    fn test_sqlx_filters_mysql() -> Result<(), AppError> {
        let filters = SqlxFilters::new("id", 1);
        let bn = BackendName::Mysql;

        assert_eq!(&filters.to_query(&bn)?, "id = ?");

        let filters = filters.add("AND age").add("OR name");
        assert_eq!(&filters.to_query(&bn)?, "id = ? AND age = ? OR name = ?");

        Ok(())
    }

    #[test]
    fn test_sqlx_values_mysql() {
        let values = SqlxValues(1, 1);
        let bn = BackendName::Mysql;

        assert_eq!(&values.to_query(&bn), "VALUES(?)");

        let values = SqlxValues(3, 1);
        assert_eq!(&values.to_query(&bn), "VALUES(?, ?, ?)");
    }
}
