use crate::grammar::{parse_expr, parse_ident};
use crate::parser::{safe_loop, Parser};
use source_gen::lexer::TokenKind;
use source_gen::syntax::SyntaxKind;
use source_gen::T;

use super::parse_into_clause;

pub(crate) fn parse_execute_immediate(p: &mut Parser) {
    p.start(SyntaxKind::ExecuteImmediateStmt);
    p.expect(T![execute]);
    p.expect(T![immediate]);
    // Parse String
    if !p.eat(T![quoted_literal]) {
        parse_ident(p, 1..1);
    }
    if p.at(T![into]) {
        parse_into_clause(p, true);
    }
    if p.at(T![bulk]) {
        parse_bulk_into_clause(p);
    }
    if p.at(T![using]) {
        parse_using_clause(p);
    }
    if [T![return], T![returning]].contains(&p.current()) {
        parse_return_into_clause(p);
    }
    p.eat(T![;]);
    p.finish();
}

fn parse_using_clause(p: &mut Parser) {
    p.start(SyntaxKind::UsingClause);
    p.expect(T![using]);
    safe_loop!(p, {
        if [T![in], T![out]].contains(&p.current()) {
            p.eat(T![in]);
            p.eat(T![out]);
        }
        parse_expr(p);
        if [T![return], T![returning], T![;]].contains(&p.current()) {
            break;
        }
        p.expect(T![,]);
    });
    p.finish();
}

fn parse_return_into_clause(p: &mut Parser) {
    p.start(SyntaxKind::ReturnIntoClause);
    p.expect_one_of(&[T![return], T![returning]]);
    // Check if bulk into or normal into
    if p.at(T![bulk]) {
        parse_bulk_into_clause(p);
    } else {
        parse_into_clause(p, false);
    }
    p.finish();
}

fn parse_bulk_into_clause(p: &mut Parser) {
    p.start(SyntaxKind::BulkIntoClause);
    p.expect(T![bulk]);
    p.expect(T![collect]);
    p.expect(T![into]);
    safe_loop!(p, {
        if !p.eat(T![bind_var]) {
            parse_ident(p, 1..1);
        }
        if [T![using], T![;]].contains(&p.current()) {
            break;
        }
        p.expect(T![,]);
    });
    p.finish();
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    use crate::grammar::parse_block;

    use super::super::tests::{check, parse};
    use super::*;

    #[test]
    fn test_parse_simple_execute_immediate() {
        check(
            parse(
                r#"EXECUTE IMMEDIATE 'SELECT * FROM emp;';"#,
                parse_execute_immediate,
            ),
            expect![[r#"
Root@0..39
  ExecuteImmediateStmt@0..39
    Keyword@0..7 "EXECUTE"
    Whitespace@7..8 " "
    Keyword@8..17 "IMMEDIATE"
    Whitespace@17..18 " "
    QuotedLiteral@18..38 "'SELECT * FROM emp;'"
    Semicolon@38..39 ";"
"#]],
            vec![],
        );
    }

    #[test]
    fn parse_complex_execute_immediate() {
        check(
            parse(
                r#"DECLARE
   sql_stmt    VARCHAR2(200);
   plsql_block VARCHAR2(500);
   emp_id      NUMBER(4) := 7566;
   salary      NUMBER(7,2);
   dept_id     NUMBER(2) := 50;
   dept_name   VARCHAR2(14) := 'PERSONNEL';
   location    VARCHAR2(13) := 'DALLAS';
   emp_rec     emp%ROWTYPE;
BEGIN
   EXECUTE IMMEDIATE 'CREATE TABLE bonus (id NUMBER, amt NUMBER)';
   sql_stmt := 'INSERT INTO dept VALUES (:1, :2, :3)';
   EXECUTE IMMEDIATE sql_stmt USING dept_id, dept_name, location;
   sql_stmt := 'SELECT * FROM emp WHERE empno = :id';
   EXECUTE IMMEDIATE sql_stmt INTO emp_rec USING emp_id;
   plsql_block := 'BEGIN emp_pkg.raise_salary(:id, :amt); END;';
   EXECUTE IMMEDIATE plsql_block USING 7788, 500;
   sql_stmt := 'UPDATE emp SET sal = 2000 WHERE empno = :1
      RETURNING sal INTO :2';
   EXECUTE IMMEDIATE sql_stmt USING emp_id RETURNING INTO salary;
   EXECUTE IMMEDIATE 'DELETE FROM dept WHERE deptno = :num'
      USING dept_id;
   EXECUTE IMMEDIATE 'ALTER SESSION SET SQL_TRACE TRUE';
END;"#,
                parse_block,
            ),
            expect![[r#"
Root@0..992
  Block@0..992
    DeclareSection@0..275
      Keyword@0..7 "DECLARE"
      Whitespace@7..11 "\n   "
      IdentGroup@11..19
        Ident@11..19 "sql_stmt"
      Whitespace@19..23 "    "
      Datatype@23..36
        Keyword@23..31 "VARCHAR2"
        LParen@31..32 "("
        Integer@32..35 "200"
        RParen@35..36 ")"
      Semicolon@36..37 ";"
      Whitespace@37..41 "\n   "
      IdentGroup@41..52
        Ident@41..52 "plsql_block"
      Whitespace@52..53 " "
      Datatype@53..66
        Keyword@53..61 "VARCHAR2"
        LParen@61..62 "("
        Integer@62..65 "500"
        RParen@65..66 ")"
      Semicolon@66..67 ";"
      Whitespace@67..71 "\n   "
      IdentGroup@71..77
        Ident@71..77 "emp_id"
      Whitespace@77..83 "      "
      Datatype@83..93
        Keyword@83..89 "NUMBER"
        LParen@89..90 "("
        Integer@90..91 "4"
        RParen@91..92 ")"
        Whitespace@92..93 " "
      Assign@93..95 ":="
      Whitespace@95..96 " "
      Expression@96..100
        Integer@96..100 "7566"
      Semicolon@100..101 ";"
      Whitespace@101..105 "\n   "
      IdentGroup@105..111
        Ident@105..111 "salary"
      Whitespace@111..117 "      "
      Datatype@117..128
        Keyword@117..123 "NUMBER"
        LParen@123..124 "("
        Integer@124..125 "7"
        Comma@125..126 ","
        Integer@126..127 "2"
        RParen@127..128 ")"
      Semicolon@128..129 ";"
      Whitespace@129..133 "\n   "
      IdentGroup@133..140
        Ident@133..140 "dept_id"
      Whitespace@140..145 "     "
      Datatype@145..155
        Keyword@145..151 "NUMBER"
        LParen@151..152 "("
        Integer@152..153 "2"
        RParen@153..154 ")"
        Whitespace@154..155 " "
      Assign@155..157 ":="
      Whitespace@157..158 " "
      Expression@158..160
        Integer@158..160 "50"
      Semicolon@160..161 ";"
      Whitespace@161..165 "\n   "
      IdentGroup@165..174
        Ident@165..174 "dept_name"
      Whitespace@174..177 "   "
      Datatype@177..190
        Keyword@177..185 "VARCHAR2"
        LParen@185..186 "("
        Integer@186..188 "14"
        RParen@188..189 ")"
        Whitespace@189..190 " "
      Assign@190..192 ":="
      Whitespace@192..193 " "
      Expression@193..204
        QuotedLiteral@193..204 "'PERSONNEL'"
      Semicolon@204..205 ";"
      Whitespace@205..209 "\n   "
      IdentGroup@209..217
        Ident@209..217 "location"
      Whitespace@217..221 "    "
      Datatype@221..234
        Keyword@221..229 "VARCHAR2"
        LParen@229..230 "("
        Integer@230..232 "13"
        RParen@232..233 ")"
        Whitespace@233..234 " "
      Assign@234..236 ":="
      Whitespace@236..237 " "
      Expression@237..245
        QuotedLiteral@237..245 "'DALLAS'"
      Semicolon@245..246 ";"
      Whitespace@246..250 "\n   "
      IdentGroup@250..257
        Ident@250..257 "emp_rec"
      Whitespace@257..262 "     "
      Datatype@262..273
        IdentGroup@262..265
          Ident@262..265 "emp"
        TypeAttribute@265..273
          Percentage@265..266 "%"
          Keyword@266..273 "ROWTYPE"
      Semicolon@273..274 ";"
      Whitespace@274..275 "\n"
    Keyword@275..280 "BEGIN"
    Whitespace@280..284 "\n   "
    BlockStatement@284..351
      ExecuteImmediateStmt@284..347
        Keyword@284..291 "EXECUTE"
        Whitespace@291..292 " "
        Keyword@292..301 "IMMEDIATE"
        Whitespace@301..302 " "
        QuotedLiteral@302..346 "'CREATE TABLE bonus ( ..."
        Semicolon@346..347 ";"
      Whitespace@347..351 "\n   "
    BlockStatement@351..402
      IdentGroup@351..359
        Ident@351..359 "sql_stmt"
      Whitespace@359..360 " "
      Assign@360..362 ":="
      Whitespace@362..363 " "
      Expression@363..401
        QuotedLiteral@363..401 "'INSERT INTO dept VAL ..."
      Semicolon@401..402 ";"
    Whitespace@402..406 "\n   "
    BlockStatement@406..472
      ExecuteImmediateStmt@406..468
        Keyword@406..413 "EXECUTE"
        Whitespace@413..414 " "
        Keyword@414..423 "IMMEDIATE"
        Whitespace@423..424 " "
        IdentGroup@424..432
          Ident@424..432 "sql_stmt"
        Whitespace@432..433 " "
        UsingClause@433..467
          Keyword@433..438 "USING"
          Whitespace@438..439 " "
          IdentGroup@439..446
            Ident@439..446 "dept_id"
          Comma@446..447 ","
          Whitespace@447..448 " "
          IdentGroup@448..457
            Ident@448..457 "dept_name"
          Comma@457..458 ","
          Whitespace@458..459 " "
          Expression@459..467
            IdentGroup@459..467
              Ident@459..467 "location"
        Semicolon@467..468 ";"
      Whitespace@468..472 "\n   "
    BlockStatement@472..522
      IdentGroup@472..480
        Ident@472..480 "sql_stmt"
      Whitespace@480..481 " "
      Assign@481..483 ":="
      Whitespace@483..484 " "
      Expression@484..521
        QuotedLiteral@484..521 "'SELECT * FROM emp WH ..."
      Semicolon@521..522 ";"
    Whitespace@522..526 "\n   "
    BlockStatement@526..583
      ExecuteImmediateStmt@526..579
        Keyword@526..533 "EXECUTE"
        Whitespace@533..534 " "
        Keyword@534..543 "IMMEDIATE"
        Whitespace@543..544 " "
        IdentGroup@544..552
          Ident@544..552 "sql_stmt"
        Whitespace@552..553 " "
        IntoClause@553..566
          Keyword@553..557 "INTO"
          Whitespace@557..558 " "
          IdentGroup@558..565
            Ident@558..565 "emp_rec"
          Whitespace@565..566 " "
        UsingClause@566..578
          Keyword@566..571 "USING"
          Whitespace@571..572 " "
          Expression@572..578
            IdentGroup@572..578
              Ident@572..578 "emp_id"
        Semicolon@578..579 ";"
      Whitespace@579..583 "\n   "
    BlockStatement@583..644
      IdentGroup@583..594
        Ident@583..594 "plsql_block"
      Whitespace@594..595 " "
      Assign@595..597 ":="
      Whitespace@597..598 " "
      Expression@598..643
        QuotedLiteral@598..643 "'BEGIN emp_pkg.raise_ ..."
      Semicolon@643..644 ";"
    Whitespace@644..648 "\n   "
    BlockStatement@648..698
      ExecuteImmediateStmt@648..694
        Keyword@648..655 "EXECUTE"
        Whitespace@655..656 " "
        Keyword@656..665 "IMMEDIATE"
        Whitespace@665..666 " "
        IdentGroup@666..677
          Ident@666..677 "plsql_block"
        Whitespace@677..678 " "
        UsingClause@678..693
          Keyword@678..683 "USING"
          Whitespace@683..684 " "
          Integer@684..688 "7788"
          Comma@688..689 ","
          Whitespace@689..690 " "
          Expression@690..693
            Integer@690..693 "500"
        Semicolon@693..694 ";"
      Whitespace@694..698 "\n   "
    BlockStatement@698..783
      IdentGroup@698..706
        Ident@698..706 "sql_stmt"
      Whitespace@706..707 " "
      Assign@707..709 ":="
      Whitespace@709..710 " "
      Expression@710..782
        QuotedLiteral@710..782 "'UPDATE emp SET sal = ..."
      Semicolon@782..783 ";"
    Whitespace@783..787 "\n   "
    BlockStatement@787..853
      ExecuteImmediateStmt@787..849
        Keyword@787..794 "EXECUTE"
        Whitespace@794..795 " "
        Keyword@795..804 "IMMEDIATE"
        Whitespace@804..805 " "
        IdentGroup@805..813
          Ident@805..813 "sql_stmt"
        Whitespace@813..814 " "
        UsingClause@814..827
          Keyword@814..819 "USING"
          Whitespace@819..820 " "
          IdentGroup@820..826
            Ident@820..826 "emp_id"
          Whitespace@826..827 " "
        ReturnIntoClause@827..848
          Keyword@827..836 "RETURNING"
          Whitespace@836..837 " "
          IntoClause@837..848
            Keyword@837..841 "INTO"
            Whitespace@841..842 " "
            IdentGroup@842..848
              Ident@842..848 "salary"
        Semicolon@848..849 ";"
      Whitespace@849..853 "\n   "
    BlockStatement@853..934
      ExecuteImmediateStmt@853..930
        Keyword@853..860 "EXECUTE"
        Whitespace@860..861 " "
        Keyword@861..870 "IMMEDIATE"
        Whitespace@870..871 " "
        QuotedLiteral@871..909 "'DELETE FROM dept WHE ..."
        Whitespace@909..916 "\n      "
        UsingClause@916..929
          Keyword@916..921 "USING"
          Whitespace@921..922 " "
          Expression@922..929
            IdentGroup@922..929
              Ident@922..929 "dept_id"
        Semicolon@929..930 ";"
      Whitespace@930..934 "\n   "
    BlockStatement@934..988
      ExecuteImmediateStmt@934..987
        Keyword@934..941 "EXECUTE"
        Whitespace@941..942 " "
        Keyword@942..951 "IMMEDIATE"
        Whitespace@951..952 " "
        QuotedLiteral@952..986 "'ALTER SESSION SET SQ ..."
        Semicolon@986..987 ";"
      Whitespace@987..988 "\n"
    Keyword@988..991 "END"
    Semicolon@991..992 ";"
"#]],
            vec![],
        );
    }
}
