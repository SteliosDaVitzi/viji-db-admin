use crate::application::{AppResult, invalid};
use sqlparser::{
    ast::{Ident, Statement, VisitMut, VisitorMut},
    dialect::Dialect,
    parser::Parser,
};
use std::ops::ControlFlow;

struct EscapeBrackets;
impl VisitorMut for EscapeBrackets {
    type Break = ();
    fn pre_visit_ident(&mut self, ident: &mut Ident) -> ControlFlow<()> {
        // sqlparser 0.63 escapes backticks/double quotes but writes bracketed
        // identifiers verbatim. Supply the T-SQL escape in the serialized copy.
        if ident.quote_style == Some('[') {
            ident.value = ident.value.replace(']', "]]");
        }
        ControlFlow::Continue(())
    }
}
pub fn canonical(statement: &Statement, dialect: &dyn Dialect) -> AppResult<String> {
    let mut output = statement.clone();
    let _ = output.visit(&mut EscapeBrackets);
    let sql = output.to_string();
    let reparsed = Parser::parse_sql(dialect, &sql).map_err(|_| {
        invalid("This SQL syntax cannot be preserved safely. Simplify the statement.")
    })?;
    // AST equality ignores source spans. Fail closed for any other printer/parser
    // mismatch, including literal contents or a newly introduced statement.
    if reparsed.as_slice() != std::slice::from_ref(statement) {
        return Err(invalid(
            "This SQL syntax cannot be preserved safely. Simplify the statement.",
        ));
    }
    Ok(sql)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlparser::dialect::{MsSqlDialect, MySqlDialect};
    #[test]
    fn identifiers_and_literals_round_trip_without_introducing_statements() {
        for (dialect, sql) in [
            (
                &MsSqlDialect {} as &dyn Dialect,
                "SELECT [a]]; DROP TABLE t;--] FROM [dbo].[t]]x]",
            ),
            (
                &MySqlDialect {} as &dyn Dialect,
                "SELECT `a``; DROP TABLE t;--` FROM `db`.`t``x`",
            ),
            (
                &MsSqlDialect {} as &dyn Dialect,
                "SELECT N'quoted ''; DROP TABLE t;--' AS value",
            ),
            (
                &MySqlDialect {} as &dyn Dialect,
                "SELECT 'quoted ''; DROP TABLE t;--' AS value",
            ),
        ] {
            let original = Parser::parse_sql(dialect, sql).unwrap().remove(0);
            let serialized = canonical(&original, dialect).unwrap();
            assert_eq!(
                Parser::parse_sql(dialect, &serialized).unwrap(),
                vec![original]
            );
        }
    }
}
