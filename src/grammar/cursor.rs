use crate::{safe_loop, Parser};
use source_gen::{lexer::TokenKind, syntax::SyntaxKind, T};

use super::{parse_datatype, parse_expr, parse_ident, parse_query};

pub fn parse_cursor(p: &mut Parser) {
    p.start(SyntaxKind::CursorStmt);
    p.expect(T![cursor]);
    parse_ident(p, 1..1);
    if p.at(T!["("]) {
        parse_cursor_param_declarations(p);
    }
    if p.eat(T![return]) {
        parse_rowtype_clause(p);
    }
    if p.eat(T![is]) {
        parse_query(p, false);
    }
    p.eat(T![;]);
    p.finish();
}

fn parse_cursor_param_declarations(p: &mut Parser) {
    p.start(SyntaxKind::CursorParameterDeclarations);
    p.expect(T!["("]);
    safe_loop!(p, {
        parse_cursor_param_declaration(p);
        if !p.eat(T![,]) {
            break;
        }
    });
    p.expect(T![")"]);
    p.finish();
}

fn parse_cursor_param_declaration(p: &mut Parser) {
    p.start(SyntaxKind::CursorParameterDeclaration);
    parse_ident(p, 1..1);
    p.eat(T![in]);
    parse_datatype(p);
    if [T![:=], T![default]].contains(&p.current()) {
        p.eat(T![:=]);
        p.eat(T![default]);
        parse_expr(p);
    }
    p.finish();
}

fn parse_rowtype_clause(p: &mut Parser) {
    p.start(SyntaxKind::RowtypeClause);
    parse_ident(p, 1..2);
    if p.eat(T![%]) && !p.eat(T![rowtype]) {
        parse_datatype(p);
    }

    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::grammar::{
        parse_block,
        tests::{check, parse},
    };

    use super::parse_cursor;

    #[test]
    fn test_explicit_cursor_declaration_and_definition() {
        check(
            parse(
                "DECLARE
  CURSOR c1 RETURN departments%ROWTYPE;    -- Declare c1
 
  CURSOR c2 IS                             -- Declare and define c2
    SELECT employee_id, job_id, salary FROM employees
    WHERE salary > 2000; 
 
  CURSOR c1 RETURN departments%ROWTYPE IS  -- Define c1,
    SELECT * FROM departments              -- repeating return type
    WHERE department_id = 110;
 
  CURSOR c3 RETURN locations%ROWTYPE;      -- Declare c3
 
  CURSOR c3 IS                             -- Define c3,
    SELECT * FROM locations                -- omitting return type
    WHERE country_id = 'JP';
BEGIN
  NULL;
END;",
                parse_block,
            ),
            expect![[r#"
Root@0..605
  Block@0..605
    DeclareSection@0..587
      Keyword@0..7 "DECLARE"
      Whitespace@7..10 "\n  "
      CursorStmt@10..47
        Keyword@10..16 "CURSOR"
        Whitespace@16..17 " "
        IdentGroup@17..19
          Ident@17..19 "c1"
        Whitespace@19..20 " "
        Keyword@20..26 "RETURN"
        Whitespace@26..27 " "
        RowtypeClause@27..46
          IdentGroup@27..38
            Ident@27..38 "departments"
          Percentage@38..39 "%"
          Keyword@39..46 "ROWTYPE"
        Semicolon@46..47 ";"
      Whitespace@47..51 "    "
      Comment@51..64 "-- Declare c1"
      Whitespace@64..69 "\n \n  "
      CursorStmt@69..219
        Keyword@69..75 "CURSOR"
        Whitespace@75..76 " "
        IdentGroup@76..78
          Ident@76..78 "c2"
        Whitespace@78..79 " "
        Keyword@79..81 "IS"
        Whitespace@81..110 "                      ..."
        Comment@110..134 "-- Declare and define c2"
        Whitespace@134..139 "\n    "
        SelectStmt@139..213
          Keyword@139..145 "SELECT"
          Whitespace@145..146 " "
          SelectClause@146..174
            ColumnExpr@146..157
              IdentGroup@146..157
                Ident@146..157 "employee_id"
            Comma@157..158 ","
            Whitespace@158..159 " "
            ColumnExpr@159..165
              IdentGroup@159..165
                Ident@159..165 "job_id"
            Comma@165..166 ","
            Whitespace@166..167 " "
            ColumnExpr@167..174
              IdentGroup@167..173
                Ident@167..173 "salary"
              Whitespace@173..174 " "
          Keyword@174..178 "FROM"
          Whitespace@178..179 " "
          IdentGroup@179..188
            Ident@179..188 "employees"
          Whitespace@188..193 "\n    "
          WhereClause@193..212
            Keyword@193..198 "WHERE"
            Whitespace@198..199 " "
            Expression@199..212
              IdentGroup@199..205
                Ident@199..205 "salary"
              Whitespace@205..206 " "
              ComparisonOp@206..207 ">"
              Whitespace@207..208 " "
              Integer@208..212 "2000"
          Semicolon@212..213 ";"
        Whitespace@213..219 " \n \n  "
      CursorStmt@219..377
        Keyword@219..225 "CURSOR"
        Whitespace@225..226 " "
        IdentGroup@226..228
          Ident@226..228 "c1"
        Whitespace@228..229 " "
        Keyword@229..235 "RETURN"
        Whitespace@235..236 " "
        RowtypeClause@236..255
          IdentGroup@236..247
            Ident@236..247 "departments"
          Percentage@247..248 "%"
          Keyword@248..255 "ROWTYPE"
        Whitespace@255..256 " "
        Keyword@256..258 "IS"
        Whitespace@258..260 "  "
        Comment@260..273 "-- Define c1,"
        Whitespace@273..278 "\n    "
        SelectStmt@278..372
          Keyword@278..284 "SELECT"
          Whitespace@284..285 " "
          Asterisk@285..286 "*"
          Whitespace@286..287 " "
          Keyword@287..291 "FROM"
          Whitespace@291..292 " "
          IdentGroup@292..303
            Ident@292..303 "departments"
          Whitespace@303..317 "              "
          Comment@317..341 "-- repeating return type"
          Whitespace@341..346 "\n    "
          WhereClause@346..371
            Keyword@346..351 "WHERE"
            Whitespace@351..352 " "
            Expression@352..371
              IdentGroup@352..365
                Ident@352..365 "department_id"
              Whitespace@365..366 " "
              ComparisonOp@366..367 "="
              Whitespace@367..368 " "
              Integer@368..371 "110"
          Semicolon@371..372 ";"
        Whitespace@372..377 "\n \n  "
      CursorStmt@377..412
        Keyword@377..383 "CURSOR"
        Whitespace@383..384 " "
        IdentGroup@384..386
          Ident@384..386 "c3"
        Whitespace@386..387 " "
        Keyword@387..393 "RETURN"
        Whitespace@393..394 " "
        RowtypeClause@394..411
          IdentGroup@394..403
            Ident@394..403 "locations"
          Percentage@403..404 "%"
          Keyword@404..411 "ROWTYPE"
        Semicolon@411..412 ";"
      Whitespace@412..418 "      "
      Comment@418..431 "-- Declare c3"
      Whitespace@431..436 "\n \n  "
      CursorStmt@436..587
        Keyword@436..442 "CURSOR"
        Whitespace@442..443 " "
        IdentGroup@443..445
          Ident@443..445 "c3"
        Whitespace@445..446 " "
        Keyword@446..448 "IS"
        Whitespace@448..477 "                      ..."
        Comment@477..490 "-- Define c3,"
        Whitespace@490..495 "\n    "
        SelectStmt@495..586
          Keyword@495..501 "SELECT"
          Whitespace@501..502 " "
          Asterisk@502..503 "*"
          Whitespace@503..504 " "
          Keyword@504..508 "FROM"
          Whitespace@508..509 " "
          IdentGroup@509..518
            Ident@509..518 "locations"
          Whitespace@518..534 "                "
          Comment@534..557 "-- omitting return type"
          Whitespace@557..562 "\n    "
          WhereClause@562..585
            Keyword@562..567 "WHERE"
            Whitespace@567..568 " "
            Expression@568..585
              IdentGroup@568..578
                Ident@568..578 "country_id"
              Whitespace@578..579 " "
              ComparisonOp@579..580 "="
              Whitespace@580..581 " "
              QuotedLiteral@581..585 "'JP'"
          Semicolon@585..586 ";"
        Whitespace@586..587 "\n"
    Keyword@587..592 "BEGIN"
    Whitespace@592..595 "\n  "
    BlockStatement@595..600
      Keyword@595..599 "NULL"
      Semicolon@599..600 ";"
    Whitespace@600..601 "\n"
    Keyword@601..604 "END"
    Semicolon@604..605 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_parse_cursor_parameters() {
        check(
            parse(
                "CURSOR c (job VARCHAR2, max_sal NUMBER) IS
    SELECT last_name, first_name, (salary - max_sal) overpayment
    FROM employees
    WHERE job_id = job
    AND salary > max_sal;",
                parse_cursor,
            ),
            expect![[r#"
Root@0..175
  CursorStmt@0..175
    Keyword@0..6 "CURSOR"
    Whitespace@6..7 " "
    IdentGroup@7..8
      Ident@7..8 "c"
    Whitespace@8..9 " "
    CursorParameterDeclarations@9..39
      LParen@9..10 "("
      CursorParameterDeclaration@10..22
        IdentGroup@10..13
          Ident@10..13 "job"
        Whitespace@13..14 " "
        Datatype@14..22
          Keyword@14..22 "VARCHAR2"
      Comma@22..23 ","
      Whitespace@23..24 " "
      CursorParameterDeclaration@24..38
        IdentGroup@24..31
          Ident@24..31 "max_sal"
        Whitespace@31..32 " "
        Datatype@32..38
          Keyword@32..38 "NUMBER"
      RParen@38..39 ")"
    Whitespace@39..40 " "
    Keyword@40..42 "IS"
    Whitespace@42..47 "\n    "
    SelectStmt@47..175
      Keyword@47..53 "SELECT"
      Whitespace@53..54 " "
      SelectClause@54..112
        ColumnExpr@54..63
          IdentGroup@54..63
            Ident@54..63 "last_name"
        Comma@63..64 ","
        Whitespace@64..65 " "
        ColumnExpr@65..75
          IdentGroup@65..75
            Ident@65..75 "first_name"
        Comma@75..76 ","
        Whitespace@76..77 " "
        ColumnExpr@77..112
          LParen@77..78 "("
          Expression@78..94
            IdentGroup@78..84
              Ident@78..84 "salary"
            Whitespace@84..85 " "
            ArithmeticOp@85..86 "-"
            Whitespace@86..87 " "
            IdentGroup@87..94
              Ident@87..94 "max_sal"
          RParen@94..95 ")"
          Whitespace@95..96 " "
          Alias@96..107
            Ident@96..107 "overpayment"
          Whitespace@107..112 "\n    "
      Keyword@112..116 "FROM"
      Whitespace@116..117 " "
      IdentGroup@117..126
        Ident@117..126 "employees"
      Whitespace@126..131 "\n    "
      WhereClause@131..174
        Keyword@131..136 "WHERE"
        Whitespace@136..137 " "
        Expression@137..174
          Expression@137..154
            IdentGroup@137..143
              Ident@137..143 "job_id"
            Whitespace@143..144 " "
            ComparisonOp@144..145 "="
            Whitespace@145..146 " "
            IdentGroup@146..149
              Ident@146..149 "job"
            Whitespace@149..154 "\n    "
          LogicOp@154..157 "AND"
          Whitespace@157..158 " "
          Expression@158..174
            IdentGroup@158..164
              Ident@158..164 "salary"
            Whitespace@164..165 " "
            ComparisonOp@165..166 ">"
            Whitespace@166..167 " "
            IdentGroup@167..174
              Ident@167..174 "max_sal"
      Semicolon@174..175 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn test_cursor_param_default_value() {
        check(
            parse(
                "CURSOR c (location NUMBER DEFAULT 1700) IS
    SELECT department_name,
           last_name manager,
           city
    FROM departments, employees, locations
    WHERE location_id = location
      AND location_id = location_id
      AND department_id = department_id;",
                parse_cursor,
            ),
            expect![[r#"
Root@0..269
  CursorStmt@0..269
    Keyword@0..6 "CURSOR"
    Whitespace@6..7 " "
    IdentGroup@7..8
      Ident@7..8 "c"
    Whitespace@8..9 " "
    CursorParameterDeclarations@9..39
      LParen@9..10 "("
      CursorParameterDeclaration@10..38
        IdentGroup@10..18
          Ident@10..18 "location"
        Whitespace@18..19 " "
        Datatype@19..26
          Keyword@19..25 "NUMBER"
          Whitespace@25..26 " "
        Keyword@26..33 "DEFAULT"
        Whitespace@33..34 " "
        Integer@34..38 "1700"
      RParen@38..39 ")"
    Whitespace@39..40 " "
    Keyword@40..42 "IS"
    Whitespace@42..47 "\n    "
    SelectStmt@47..269
      Keyword@47..53 "SELECT"
      Whitespace@53..54 " "
      SelectClause@54..121
        ColumnExpr@54..69
          IdentGroup@54..69
            Ident@54..69 "department_name"
        Comma@69..70 ","
        Whitespace@70..82 "\n           "
        ColumnExpr@82..99
          IdentGroup@82..91
            Ident@82..91 "last_name"
          Whitespace@91..92 " "
          Alias@92..99
            Ident@92..99 "manager"
        Comma@99..100 ","
        Whitespace@100..112 "\n           "
        ColumnExpr@112..121
          IdentGroup@112..116
            Ident@112..116 "city"
          Whitespace@116..121 "\n    "
      Keyword@121..125 "FROM"
      Whitespace@125..126 " "
      IdentGroup@126..137
        Ident@126..137 "departments"
      Comma@137..138 ","
      Whitespace@138..139 " "
      IdentGroup@139..148
        Ident@139..148 "employees"
      Comma@148..149 ","
      Whitespace@149..150 " "
      IdentGroup@150..159
        Ident@150..159 "locations"
      Whitespace@159..164 "\n    "
      WhereClause@164..268
        Keyword@164..169 "WHERE"
        Whitespace@169..170 " "
        Expression@170..268
          Expression@170..235
            Expression@170..199
              IdentGroup@170..181
                Ident@170..181 "location_id"
              Whitespace@181..182 " "
              ComparisonOp@182..183 "="
              Whitespace@183..184 " "
              IdentGroup@184..192
                Ident@184..192 "location"
              Whitespace@192..199 "\n      "
            LogicOp@199..202 "AND"
            Whitespace@202..203 " "
            Expression@203..235
              IdentGroup@203..214
                Ident@203..214 "location_id"
              Whitespace@214..215 " "
              ComparisonOp@215..216 "="
              Whitespace@216..217 " "
              IdentGroup@217..228
                Ident@217..228 "location_id"
              Whitespace@228..235 "\n      "
          LogicOp@235..238 "AND"
          Whitespace@238..239 " "
          Expression@239..268
            IdentGroup@239..252
              Ident@239..252 "department_id"
            Whitespace@252..253 " "
            ComparisonOp@253..254 "="
            Whitespace@254..255 " "
            IdentGroup@255..268
              Ident@255..268 "department_id"
      Semicolon@268..269 ";"
"#]],
            vec![],
        );
    }
}
