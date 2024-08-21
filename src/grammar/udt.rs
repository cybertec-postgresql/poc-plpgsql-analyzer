use crate::{
    grammar::{element_spec::parse_element_spec, parse_datatype},
    safe_loop, Parser,
};
use source_gen::{lexer::TokenKind, syntax::SyntaxKind, T};

use super::{call_spec::opt_call_spec, parse_block, parse_expr, parse_ident, parse_param_list};

#[allow(unused)]
pub(crate) fn parse_udt(p: &mut Parser) {
    p.start(SyntaxKind::UdtDefinitionStmt);
    p.expect(T![create]);
    if p.eat(T![or]) {
        p.expect(T![replace]);
    }
    p.eat_one_of(&[T![editionable], T![noneditionable]]);
    p.expect(T![type]);
    let is_body = p.eat(T![body]);
    if p.eat(T![if]) {
        p.expect(T![not]);
        p.expect(T![exists]);
    }
    if is_body {
        parse_plsql_type_body_source(p);
    } else {
        parse_plsql_type_source(p);
    }
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

fn parse_plsql_type_body_source(p: &mut Parser) {
    p.start(SyntaxKind::PlsqlBodyTypeSource);
    parse_ident(p, 1..2);
    if p.at(T![sharing]) {
        parse_sharing_clause(p);
    }
    p.expect_one_of(&[T![is], T![as]]);
    safe_loop!(p, {
        match p.current() {
            T![map] | T![order] | T![member] => parse_map_order_func_declaration(p),
            _ => parse_subprog_decl_in_type(p),
        }
        if !p.eat(T![,]) {
            break;
        }
    });
    p.expect(T![end]);
    p.finish();
}

fn parse_map_order_func_declaration(p: &mut Parser) {
    p.start(SyntaxKind::MapOrderFuncDeclaration);
    p.eat_one_of(&[T![map], T![order]]);
    p.expect(T![member]);
    parse_func_decl_in_type(p);
    p.finish();
}

fn parse_subprog_decl_in_type(p: &mut Parser) {
    p.start(SyntaxKind::SubprogDeclInType);
    match p.current() {
        T![function] => parse_func_decl_in_type(p),
        T![procedure] => parse_proc_decl_in_type(p),
        _ => parse_constructor_declaration(p),
    }
    p.finish();
}

fn parse_func_decl_in_type(p: &mut Parser) {
    p.start(SyntaxKind::FuncDeclInType);
    p.expect(T![function]);
    parse_ident(p, 1..2);
    parse_param_list(p);
    p.expect(T![return]);
    parse_datatype(p);
    safe_loop!(p, {
        match p.current() {
            T![deterministic] => {
                p.expect(T![deterministic]);
            }
            T![accessible] => parse_accessible_by_clause(p),
            T![authid] => parse_invoker_rights_clause(p),
            T![result_cache] => parse_result_cache_clause(p),
            T![parallel_enable] => parse_parallel_enable_clause(p),
            _ => (),
        }
        if p.at(T![pipelined]) || p.at(T![as]) || p.at(T![is]) {
            break;
        }
    });
    p.eat(T![pipelined]);
    p.expect_one_of(&[T![is], T![as]]);
    if !opt_call_spec(p) {
        parse_block(p);
    }
    p.finish();
}

fn parse_proc_decl_in_type(p: &mut Parser) {
    p.start(SyntaxKind::ProcDeclInType);
    p.expect(T![procedure]);
    parse_ident(p, 1..2);
    parse_param_list(p);
    p.expect_one_of(&[T![is], T![as]]);
    if !opt_call_spec(p) {
        parse_block(p);
    }

    p.finish();
}

fn parse_constructor_declaration(p: &mut Parser) {
    p.start(SyntaxKind::ConstructorDeclaration);
    p.eat(T![final]);
    p.eat(T![instantiable]);
    p.expect(T![constructor]);
    p.expect(T![function]);
    parse_datatype(p);
    if p.eat(T!["("]) {
        if p.eat(T![self]) {
            p.expect(T![in]);
            p.expect(T![out]);
            parse_datatype(p);
            p.expect(T![,]);
        }
        safe_loop!(p, {
            parse_ident(p, 1..1);
            parse_datatype(p);
            if !p.eat(T!["("]) {
                break;
            }
        });
        p.expect(T![")"]);
    }
    p.expect(T![return]);
    p.expect(T![self]);
    p.expect(T![as]);
    p.expect(T![result]);
    p.expect_one_of(&[T![is], T![as]]);
    if !opt_call_spec(p) {
        parse_block(p);
    }

    p.finish();
}

fn parse_result_cache_clause(p: &mut Parser) {
    p.start(SyntaxKind::ResultCacheClause);
    p.expect(T![result_cache]);
    if p.eat(T![relies_on]) {
        p.expect(T!["("]);
        safe_loop!(p, {
            parse_ident(p, 1..2);
            if !p.eat(T![,]) {
                break;
            }
        });
        p.expect(T![")"]);
    }
    p.finish();
}

fn parse_parallel_enable_clause(p: &mut Parser) {
    p.start(SyntaxKind::ParallelEnableClause);
    p.expect(T![parallel_enable]);
    if p.eat(T!["("]) {
        p.expect(T![partition]);
        parse_ident(p, 1..1);
        p.expect(T![by]);
        match p.current() {
            T![any] => {
                p.expect(T![any]);
            }
            T![hash] | T![range] => {
                p.expect_one_of(&[T![hash], T![range]]);
                p.expect(T!["("]);
                safe_loop!(p, {
                    parse_ident(p, 1..1);
                    if !p.eat(T![,]) {
                        break;
                    }
                });
                if [T![order], T![cluster]].contains(&p.current()) {
                    parse_streaming_clause(p);
                }
                p.expect(T![")"]);
            }
            T![value] => {
                p.expect(T!["("]);
                parse_ident(p, 1..1);
                p.expect(T![")"]);
            }
            _ => (),
        }
        p.expect(T![")"]);
    }
    p.finish();
}

fn parse_streaming_clause(p: &mut Parser) {
    p.start(SyntaxKind::StreamingClause);
    p.expect_one_of(&[T![cluster], T![order]]);
    parse_expr(p);
    p.expect(T![by]);
    p.expect(T!["("]);
    safe_loop!(p, {
        parse_ident(p, 1..3);
        if !p.eat(T![,]) {
            break;
        }
    });
    p.expect(T![")"]);

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
    if p.eat(T!["("]) {
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
        p.expect(T![")"]);
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

    if p.eat(T!["("]) {
        safe_loop!(p, {
            parse_ident(p, 1..1);
            parse_datatype(p);
            if !p.eat(T![,]) || p.at(T![")"]) {
                break;
            }
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
        p.expect(T![")"]);
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
    let expect_rparen = p.eat(T!["("]);
    parse_datatype(p);
    if p.eat(T![not]) {
        p.expect(T![null]);
    }
    if expect_rparen {
        p.expect(T![")"]);
    }
    p.eat(T![not]);
    p.eat(T![persistable]);
    p.finish();
}

fn parse_nested_table_type_spec(p: &mut Parser) {
    p.start(SyntaxKind::NestedTableTypeSpec);
    p.expect(T![table]);
    p.expect(T![of]);
    let expect_rparen = p.eat(T!["("]);
    parse_datatype(p);
    if p.eat(T![not]) {
        p.expect(T![null]);
    }
    if expect_rparen {
        p.expect(T![")"]);
    }
    p.eat(T![not]);
    p.eat(T![persistable]);
    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::grammar::tests::{check, parse};

    use super::parse_udt;

    #[test]
    fn simple_udt() {
        check(
            parse(
                "CREATE TYPE customer_typ_demo AS OBJECT ( customer_id NUMBER(6)
    , cust_address       CUST_ADDRESS_TYP
    ) ;",
                parse_udt,
            ),
            expect![[r#"
Root@0..419
  UdtDefinitionStmt@0..419
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..11 "TYPE"
    Whitespace@11..12 " "
    PlsqlTypeSource@12..418
      IdentGroup@12..29
        Ident@12..29 "customer_typ_demo"
      Whitespace@29..30 " "
      ObjectBaseTypeDef@30..418
        Keyword@30..32 "AS"
        Whitespace@32..33 " "
        ObjectTypeDef@33..418
          Keyword@33..39 "OBJECT"
          Whitespace@39..40 " "
          LParen@40..41 "("
          Whitespace@41..42 " "
          IdentGroup@42..53
            Ident@42..53 "customer_id"
          Whitespace@53..54 " "
          Datatype@54..68
            Keyword@54..60 "NUMBER"
            LParen@60..61 "("
            Integer@61..62 "6"
            RParen@62..63 ")"
            Whitespace@63..68 "\n    "
          Comma@68..69 ","
          Whitespace@69..70 " "
          IdentGroup@70..85
            Ident@70..85 "cust_first_name"
          Whitespace@85..89 "    "
          Datatype@89..106
            Keyword@89..97 "VARCHAR2"
            LParen@97..98 "("
            Integer@98..100 "20"
            RParen@100..101 ")"
            Whitespace@101..106 "\n    "
          Comma@106..107 ","
          Whitespace@107..108 " "
          IdentGroup@108..122
            Ident@108..122 "cust_last_name"
          Whitespace@122..127 "     "
          Datatype@127..144
            Keyword@127..135 "VARCHAR2"
            LParen@135..136 "("
            Integer@136..138 "20"
            RParen@138..139 ")"
            Whitespace@139..144 "\n    "
          Comma@144..145 ","
          Whitespace@145..146 " "
          IdentGroup@146..158
            Ident@146..158 "cust_address"
          Whitespace@158..165 "       "
          Datatype@165..186
            IdentGroup@165..181
              Ident@165..181 "CUST_ADDRESS_TYP"
            Whitespace@181..186 "\n    "
          Comma@186..187 ","
          Whitespace@187..188 " "
          IdentGroup@188..201
            Ident@188..201 "phone_numbers"
          Whitespace@201..207 "      "
          Datatype@207..226
            IdentGroup@207..221
              Ident@207..221 "PHONE_LIST_TYP"
            Whitespace@221..226 "\n    "
          Comma@226..227 ","
          Whitespace@227..228 " "
          IdentGroup@228..240
            Ident@228..240 "nls_language"
          Whitespace@240..247 "       "
          Datatype@247..263
            Keyword@247..255 "VARCHAR2"
            LParen@255..256 "("
            Integer@256..257 "3"
            RParen@257..258 ")"
            Whitespace@258..263 "\n    "
          Comma@263..264 ","
          Whitespace@264..265 " "
          IdentGroup@265..278
            Ident@265..278 "nls_territory"
          Whitespace@278..284 "      "
          Datatype@284..301
            Keyword@284..292 "VARCHAR2"
            LParen@292..293 "("
            Integer@293..295 "30"
            RParen@295..296 ")"
            Whitespace@296..301 "\n    "
          Comma@301..302 ","
          Whitespace@302..303 " "
          IdentGroup@303..315
            Ident@303..315 "credit_limit"
          Whitespace@315..322 "       "
          Datatype@322..338
            Keyword@322..328 "NUMBER"
            LParen@328..329 "("
            Integer@329..330 "9"
            Comma@330..331 ","
            Integer@331..332 "2"
            RParen@332..333 ")"
            Whitespace@333..338 "\n    "
          Comma@338..339 ","
          Whitespace@339..340 " "
          IdentGroup@340..350
            Ident@340..350 "cust_email"
          Whitespace@350..359 "         "
          Datatype@359..376
            Keyword@359..367 "VARCHAR2"
            LParen@367..368 "("
            Integer@368..370 "30"
            RParen@370..371 ")"
            Whitespace@371..376 "\n    "
          Comma@376..377 ","
          Whitespace@377..378 " "
          IdentGroup@378..389
            Ident@378..389 "cust_orders"
          Whitespace@389..397 "        "
          Datatype@397..416
            IdentGroup@397..411
              Ident@397..411 "ORDER_LIST_TYP"
            Whitespace@411..416 "\n    "
          RParen@416..417 ")"
          Whitespace@417..418 " "
    Semicolon@418..419 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_simple_udt2() {
        check(
            parse(
                "CREATE TYPE cust_address_typ2 AS OBJECT
       ( street_address     VARCHAR2(40)
       , postal_code        VARCHAR2(10)
       , city               VARCHAR2(30)
       , state_province     VARCHAR2(10)
       , country_id         CHAR(2)
       , phone              phone_list_typ_demo
       );",
                parse_udt,
            ),
            expect![[r#"
Root@0..297
  UdtDefinitionStmt@0..297
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..11 "TYPE"
    Whitespace@11..12 " "
    PlsqlTypeSource@12..296
      IdentGroup@12..29
        Ident@12..29 "cust_address_typ2"
      Whitespace@29..30 " "
      ObjectBaseTypeDef@30..296
        Keyword@30..32 "AS"
        Whitespace@32..33 " "
        ObjectTypeDef@33..296
          Keyword@33..39 "OBJECT"
          Whitespace@39..47 "\n       "
          LParen@47..48 "("
          Whitespace@48..49 " "
          IdentGroup@49..63
            Ident@49..63 "street_address"
          Whitespace@63..68 "     "
          Datatype@68..88
            Keyword@68..76 "VARCHAR2"
            LParen@76..77 "("
            Integer@77..79 "40"
            RParen@79..80 ")"
            Whitespace@80..88 "\n       "
          Comma@88..89 ","
          Whitespace@89..90 " "
          IdentGroup@90..101
            Ident@90..101 "postal_code"
          Whitespace@101..109 "        "
          Datatype@109..129
            Keyword@109..117 "VARCHAR2"
            LParen@117..118 "("
            Integer@118..120 "10"
            RParen@120..121 ")"
            Whitespace@121..129 "\n       "
          Comma@129..130 ","
          Whitespace@130..131 " "
          IdentGroup@131..135
            Ident@131..135 "city"
          Whitespace@135..150 "               "
          Datatype@150..170
            Keyword@150..158 "VARCHAR2"
            LParen@158..159 "("
            Integer@159..161 "30"
            RParen@161..162 ")"
            Whitespace@162..170 "\n       "
          Comma@170..171 ","
          Whitespace@171..172 " "
          IdentGroup@172..186
            Ident@172..186 "state_province"
          Whitespace@186..191 "     "
          Datatype@191..211
            Keyword@191..199 "VARCHAR2"
            LParen@199..200 "("
            Integer@200..202 "10"
            RParen@202..203 ")"
            Whitespace@203..211 "\n       "
          Comma@211..212 ","
          Whitespace@212..213 " "
          IdentGroup@213..223
            Ident@213..223 "country_id"
          Whitespace@223..232 "         "
          Datatype@232..247
            Keyword@232..236 "CHAR"
            LParen@236..237 "("
            Integer@237..238 "2"
            RParen@238..239 ")"
            Whitespace@239..247 "\n       "
          Comma@247..248 ","
          Whitespace@248..249 " "
          IdentGroup@249..254
            Ident@249..254 "phone"
          Whitespace@254..268 "              "
          Datatype@268..295
            IdentGroup@268..287
              Ident@268..287 "phone_list_typ_demo"
            Whitespace@287..295 "\n       "
          RParen@295..296 ")"
    Semicolon@296..297 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_object_sub_type() {
        check(
            parse(
                "CREATE TYPE corporate_customer_typ_demo UNDER customer_typ
    ( account_mgr_id     NUMBER(6)
    );",
                parse_udt,
            ),
            expect![[r#"
Root@0..100
  UdtDefinitionStmt@0..100
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..11 "TYPE"
    Whitespace@11..12 " "
    PlsqlTypeSource@12..99
      IdentGroup@12..39
        Ident@12..39 "corporate_customer_ty ..."
      Whitespace@39..40 " "
      ObjectSubtypeDef@40..99
        Keyword@40..45 "UNDER"
        Whitespace@45..46 " "
        IdentGroup@46..58
          Ident@46..58 "customer_typ"
        Whitespace@58..63 "\n    "
        LParen@63..64 "("
        Whitespace@64..65 " "
        IdentGroup@65..79
          Ident@65..79 "account_mgr_id"
        Whitespace@79..84 "     "
        Datatype@84..98
          Keyword@84..90 "NUMBER"
          LParen@90..91 "("
          Integer@91..92 "6"
          RParen@92..93 ")"
          Whitespace@93..98 "\n    "
        RParen@98..99 ")"
    Semicolon@99..100 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_member_type() {
        check(
            parse(
                "CREATE TYPE data_typ1 AS OBJECT 
   ( year NUMBER, 
     MEMBER FUNCTION prod(invent NUMBER) RETURN NUMBER 
   );",
                parse_udt,
            ),
            expect![[r#"
Root@0..113
  UdtDefinitionStmt@0..113
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..11 "TYPE"
    Whitespace@11..12 " "
    PlsqlTypeSource@12..112
      IdentGroup@12..21
        Ident@12..21 "data_typ1"
      Whitespace@21..22 " "
      ObjectBaseTypeDef@22..112
        Keyword@22..24 "AS"
        Whitespace@24..25 " "
        ObjectTypeDef@25..112
          Keyword@25..31 "OBJECT"
          Whitespace@31..36 " \n   "
          LParen@36..37 "("
          Whitespace@37..38 " "
          IdentGroup@38..42
            Ident@38..42 "year"
          Whitespace@42..43 " "
          Datatype@43..49
            Keyword@43..49 "NUMBER"
          Comma@49..50 ","
          Whitespace@50..57 " \n     "
          ElementSpec@57..111
            Keyword@57..63 "MEMBER"
            Whitespace@63..64 " "
            FunctionSpec@64..111
              Keyword@64..72 "FUNCTION"
              Whitespace@72..73 " "
              IdentGroup@73..77
                Ident@73..77 "prod"
              LParen@77..78 "("
              IdentGroup@78..84
                Ident@78..84 "invent"
              Whitespace@84..85 " "
              Datatype@85..91
                Keyword@85..91 "NUMBER"
              RParen@91..92 ")"
              Whitespace@92..93 " "
              Keyword@93..99 "RETURN"
              Whitespace@99..100 " "
              Datatype@100..111
                Keyword@100..106 "NUMBER"
                Whitespace@106..111 " \n   "
          RParen@111..112 ")"
    Semicolon@112..113 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_varray_udt() {
        check(
            parse(
                "CREATE TYPE phone_list_typ_demo AS VARRAY(5) OF VARCHAR2(25);",
                parse_udt,
            ),
            expect![[r#"
Root@0..61
  UdtDefinitionStmt@0..61
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..11 "TYPE"
    Whitespace@11..12 " "
    PlsqlTypeSource@12..60
      IdentGroup@12..31
        Ident@12..31 "phone_list_typ_demo"
      Whitespace@31..32 " "
      ObjectBaseTypeDef@32..60
        Keyword@32..34 "AS"
        Whitespace@34..35 " "
        VarrayTypeSpec@35..60
          Keyword@35..41 "VARRAY"
          LParen@41..42 "("
          Integer@42..43 "5"
          RParen@43..44 ")"
          Whitespace@44..45 " "
          Keyword@45..47 "OF"
          Whitespace@47..48 " "
          Datatype@48..60
            Keyword@48..56 "VARCHAR2"
            LParen@56..57 "("
            Integer@57..59 "25"
            RParen@59..60 ")"
    Semicolon@60..61 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_non_persist_varray_udt() {
        check(
            parse(
                "CREATE TYPE IF NOT EXISTS varr_int AS VARRAY(10) OF (PLS_INTEGER) NOT PERSISTABLE;", 
                parse_udt
            ), expect![[r#"
Root@0..82
  UdtDefinitionStmt@0..82
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..11 "TYPE"
    Whitespace@11..12 " "
    Keyword@12..14 "IF"
    Whitespace@14..15 " "
    Keyword@15..18 "NOT"
    Whitespace@18..19 " "
    Keyword@19..25 "EXISTS"
    Whitespace@25..26 " "
    PlsqlTypeSource@26..81
      IdentGroup@26..34
        Ident@26..34 "varr_int"
      Whitespace@34..35 " "
      ObjectBaseTypeDef@35..81
        Keyword@35..37 "AS"
        Whitespace@37..38 " "
        VarrayTypeSpec@38..81
          Keyword@38..44 "VARRAY"
          LParen@44..45 "("
          Integer@45..47 "10"
          RParen@47..48 ")"
          Whitespace@48..49 " "
          Keyword@49..51 "OF"
          Whitespace@51..52 " "
          LParen@52..53 "("
          Datatype@53..64
            Keyword@53..64 "PLS_INTEGER"
          RParen@64..65 ")"
          Whitespace@65..66 " "
          Keyword@66..69 "NOT"
          Whitespace@69..70 " "
          Keyword@70..81 "PERSISTABLE"
    Semicolon@81..82 ";"
"#]], 
    vec![]);
    }

    #[test]
    fn test_nested_table_udt() {
        check(
            parse(
                "CREATE TYPE textdoc_tab AS TABLE OF textdoc_typ;",
                parse_udt,
            ),
            expect![[r#"
Root@0..48
  UdtDefinitionStmt@0..48
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..11 "TYPE"
    Whitespace@11..12 " "
    PlsqlTypeSource@12..47
      IdentGroup@12..23
        Ident@12..23 "textdoc_tab"
      Whitespace@23..24 " "
      ObjectBaseTypeDef@24..47
        Keyword@24..26 "AS"
        Whitespace@26..27 " "
        NestedTableTypeSpec@27..47
          Keyword@27..32 "TABLE"
          Whitespace@32..33 " "
          Keyword@33..35 "OF"
          Whitespace@35..36 " "
          Datatype@36..47
            IdentGroup@36..47
              Ident@36..47 "textdoc_typ"
    Semicolon@47..48 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_udt_body() {
        check(
            parse(
                "CREATE OR REPLACE TYPE BODY employee_t IS
   FUNCTION construct_emp
   (name varchar2, dept department_t)
   RETURN employee_t IS
      BEGIN
         return employee_t(SYS_GUID(),name,dept);
      END;
END;",
                parse_udt,
            ),
            expect![[r#"
Root@0..207
  UdtDefinitionStmt@0..207
    Keyword@0..6 "CREATE"
    Whitespace@6..7 " "
    Keyword@7..9 "OR"
    Whitespace@9..10 " "
    Keyword@10..17 "REPLACE"
    Whitespace@17..18 " "
    Keyword@18..22 "TYPE"
    Whitespace@22..23 " "
    Keyword@23..27 "BODY"
    Whitespace@27..28 " "
    PlsqlBodyTypeSource@28..206
      IdentGroup@28..38
        Ident@28..38 "employee_t"
      Whitespace@38..39 " "
      Keyword@39..41 "IS"
      Whitespace@41..45 "\n   "
      SubprogDeclInType@45..203
        FuncDeclInType@45..203
          Keyword@45..53 "FUNCTION"
          Whitespace@53..54 " "
          IdentGroup@54..67
            Ident@54..67 "construct_emp"
          Whitespace@67..71 "\n   "
          ParamList@71..105
            LParen@71..72 "("
            Param@72..85
              IdentGroup@72..76
                Ident@72..76 "name"
              Whitespace@76..77 " "
              Datatype@77..85
                Keyword@77..85 "varchar2"
            Comma@85..86 ","
            Whitespace@86..87 " "
            Param@87..104
              IdentGroup@87..91
                Ident@87..91 "dept"
              Whitespace@91..92 " "
              Datatype@92..104
                IdentGroup@92..104
                  Ident@92..104 "department_t"
            RParen@104..105 ")"
          Whitespace@105..109 "\n   "
          Keyword@109..115 "RETURN"
          Whitespace@115..116 " "
          Datatype@116..127
            IdentGroup@116..126
              Ident@116..126 "employee_t"
            Whitespace@126..127 " "
          Keyword@127..129 "IS"
          Whitespace@129..136 "\n      "
          Block@136..202
            Keyword@136..141 "BEGIN"
            Whitespace@141..151 "\n         "
            BlockStatement@151..191
              Keyword@151..157 "return"
              Whitespace@157..158 " "
              Expression@158..190
                FunctionInvocation@158..190
                  IdentGroup@158..168
                    Ident@158..168 "employee_t"
                  LParen@168..169 "("
                  ArgumentList@169..189
                    Argument@169..179
                      Expression@169..179
                        FunctionInvocation@169..179
                          IdentGroup@169..177
                            Ident@169..177 "SYS_GUID"
                          LParen@177..178 "("
                          RParen@178..179 ")"
                    Comma@179..180 ","
                    Argument@180..184
                      Expression@180..184
                        IdentGroup@180..184
                          Ident@180..184 "name"
                    Comma@184..185 ","
                    Argument@185..189
                      IdentGroup@185..189
                        Ident@185..189 "dept"
                  RParen@189..190 ")"
              Semicolon@190..191 ";"
            Whitespace@191..198 "\n      "
            Keyword@198..201 "END"
            Semicolon@201..202 ";"
          Whitespace@202..203 "\n"
      Keyword@203..206 "END"
    Semicolon@206..207 ";"
"#]],
            vec![],
        );
    }
}
