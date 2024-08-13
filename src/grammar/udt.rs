use crate::{
    grammar::{element_spec::parse_element_spec, parse_datatype},
    safe_loop, Parser,
};
use source_gen::{lexer::TokenKind, syntax::SyntaxKind, T};

use super::parse_ident;

#[allow(unused)]
pub(crate) fn parse_create_type(p: &mut Parser) {
    p.start(SyntaxKind::UdtDefinitionStmt);
    p.expect(T![create]);
    if p.eat(T![or]) {
        p.expect(T![replace]);
    }
    p.eat_one_of(&[T![editionable], T![noneditionable]]);
    p.expect(T![type]);
    if p.eat(T![if]) {
        p.expect(T![not]);
        p.expect(T![exists]);
    }
    parse_plsql_type_source(p);
    p.eat(T![;]);
    p.finish();
}

fn parse_plsql_type_source(p: &mut Parser) {
    p.start(SyntaxKind::PlsqlTypeSource);
    parse_ident(p, 1..2);
    p.eat(T![force]);
    if p.eat(T![oid]) {
        p.expect(T![quoted_ident]);
    }
    if p.at(T![sharing]) {
        parse_sharing_clause(p);
    }
    if p.at(T![default]) {
        parse_default_collation_clause(p);
    }
    safe_loop!(p, {
        if p.current() == T![accessible] {
            parse_accessible_by_clause(p);
        } else if p.current() == T![authid] {
            parse_invoker_rights_clause(p);
        } else {
            break;
        }

        if [T![is], T![as], T![under]].contains(&p.current()) {
            break;
        }
    });
    if [T![is], T![as]].contains(&p.current()) {
        parse_object_base_type_def(p);
    } else if p.at(T![under]) {
        parse_object_subtype_def(p);
    }
    p.finish();
}

fn parse_sharing_clause(p: &mut Parser) {
    p.start(SyntaxKind::SharingClause);
    p.expect(T![sharing]);
    p.expect(T![=]);
    p.expect_one_of(&[T![metadata], T![none]]);
    p.finish();
}

fn parse_default_collation_clause(p: &mut Parser) {
    p.start(SyntaxKind::DefaultCollationClause);
    p.expect(T![default]);
    p.expect(T![collation]);
    p.expect(T![using_nls_comp]);
    p.finish();
}

fn parse_accessible_by_clause(p: &mut Parser) {
    p.start(SyntaxKind::AccessibleByClause);
    p.expect(T![accessible]);
    p.expect(T![by]);
    p.expect(T!["("]);
    safe_loop!(p, {
        p.eat_one_of(&[
            T![function],
            T![procedure],
            T![package],
            T![trigger],
            T![type],
        ]);
        parse_ident(p, 1..2);
        if !p.at(T![,]) || p.at(T![")"]) {
            break;
        }
    });
    p.expect(T![")"]);
    p.finish();
}

fn parse_invoker_rights_clause(p: &mut Parser) {
    p.start(SyntaxKind::InvokerRightsClause);
    p.expect(T![authid]);
    p.expect_one_of(&[T![current_user], T![definer]]);
    p.finish();
}

fn parse_object_base_type_def(p: &mut Parser) {
    p.start(SyntaxKind::ObjectBaseTypeDef);
    p.expect_one_of(&[T![is], T![as]]);
    match p.current() {
        T![object] => parse_object_type_def(p),
        T![varray] | T![varying] => parse_varray_type_spec(p),
        T![table] => parse_nested_table_type_spec(p),
        _ => (),
    }
    p.finish();
}

fn parse_object_subtype_def(p: &mut Parser) {
    p.start(SyntaxKind::ObjectSubtypeDef);
    p.expect(T![under]);
    parse_ident(p, 1..2);
    if p.at(T!["("]) {
        safe_loop!(p, {
            parse_ident(p, 1..1);
            parse_datatype(p);
            if !p.at(T![,]) || p.at(T![")"]) {
                break;
            }
        });
        if [
            T![constructor],
            T![final],
            T![instantiable],
            T![map],
            T![member],
            T![not],
            T![order],
            T![overriding],
            T![static],
        ]
        .contains(&p.current())
        {
            safe_loop!(p, {
                parse_element_spec(p);
                if !p.at(T![,]) || p.at(T![")"]) {
                    break;
                }
            });
        }
    }
    safe_loop!(p, {
        let mut ate_something = false;
        ate_something |= p.eat(T![not]);
        ate_something |= p.eat_one_of(&[T![final], T![instantiable]]);
        if p.at(T![;]) || !ate_something {
            break;
        }
    });
    p.finish();
}

fn parse_object_type_def(p: &mut Parser) {
    p.start(SyntaxKind::ObjectTypeDef);
    p.expect(T![object]);

    if p.at(T!["("]) {
        safe_loop!(p, {
            parse_ident(p, 1..1);
            parse_datatype(p);
            if !p.at(T![,]) || p.at(T![")"]) {
                break;
            }
        });
        if [
            T![constructor],
            T![final],
            T![instantiable],
            T![map],
            T![member],
            T![not],
            T![order],
            T![overriding],
            T![static],
        ]
        .contains(&p.current())
        {
            safe_loop!(p, {
                parse_element_spec(p);
                if !p.at(T![,]) || p.at(T![")"]) {
                    break;
                }
            });
        }
    }
    safe_loop!(p, {
        let mut ate_something = false;
        ate_something |= p.eat(T![not]);
        ate_something |= p.eat_one_of(&[T![final], T![instantiable], T![persistable]]);
        if p.at(T![;]) || !ate_something {
            break;
        }
    });

    p.finish();
}

fn parse_varray_type_spec(p: &mut Parser) {
    p.start(SyntaxKind::VarrayTypeSpec);
    if p.eat(T![varying]) {
        p.expect(T![array]);
    } else {
        p.expect(T![varray]);
    }
    p.expect(T!["("]);
    p.expect(T![int_literal]);
    p.expect(T![")"]);
    p.expect(T![of]);
    p.expect(T!["("]);
    parse_datatype(p);
    if p.eat(T![not]) {
        p.expect(T![null]);
    }
    p.expect(T![")"]);
    p.eat(T![not]);
    p.eat(T![persistable]);
    p.finish();
}

fn parse_nested_table_type_spec(p: &mut Parser) {
    p.start(SyntaxKind::NestedTableTypeSpec);
    p.expect(T![table]);
    p.expect(T![of]);
    p.expect(T!["("]);
    parse_datatype(p);
    if p.eat(T![not]) {
        p.expect(T![null]);
    }
    p.expect(T![")"]);
    p.eat(T![not]);
    p.eat(T![persistable]);
    p.finish();
}
