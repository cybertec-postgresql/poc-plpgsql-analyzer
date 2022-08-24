# poc-plpgsql-analyzer
Proof of concept for tooling framework to migrate PL/SQL code to PL/pgSQL written in Rust.

We want to achieve following goals
* extract the information from the DDL code of a database object (DBO) to assess the migration effort to PostgreSQL
* transpile Oracle PL/SQL code to PL/pgSQL
* show language constructs which can't be transpiled to PL/pgSQL

The functionality is used in browsers (WASM+JavaScript) but also in applications written in other languages (C, Rust, TypeScript).

> **Note**  
> The tooling should support multiple RDBMS (Oracle, DB2, Informix, MS SQL Server, ...) and versions.
> As starting point we focus on Oracle but we should keep in mind the extendability for other RDBMS.

We need a PL/SQL parser library to convert (PL/)SQL into an abstract syntax tree (AST).

## Table of Contents
1. [Problem Description](#problem-description)
    1. [Example](#example)
    2. [Migration Assessment](#migration-assessment)
    3. [Browser and WASM](#browser-and-wasm)
    4. [Transpiler to PL/pgSQL](#transpiler-to-plpgsql)
2. [Desired Outcome and Service Provision](#desired-outcomes-and-service-provision)
3. [Questions to solve](#questions-to-solve)
3. [References](#references)

## Problem Description
Given a (valid) SQL DDL command to `CREATE` a DBO extract its metadata.
The extracted metadata will be used to calculate a numerical value (in an abstract unit) for the effort to migrate the DBO to PostgreSQL.

DBOs which may take considerable effort to migrate to PostgreSQL are
* views
* functions
* user defined procedures (UDP)
* triggers
* packages

Apart of the DBOs listed above we have to assess the effort to migrate **other language constructs** to PostgreSQL
* column check constraint expressions
* column default expressions
* expressions in functional indexes

The **error handling** should be permissive. 
If the analyzer can't understand a language construct it should provide a descriptive error output and try to recover as fast as possible with the goal to analyze the rest.

### Example
We will use an UDP from the Oracle `HR` example database to show the changes a user has to apply to migrate it to PostgreSQL.

```sql
-- Oracle PL/SQL
CREATE PROCEDURE secure_dml
IS
BEGIN
  IF TO_CHAR (SYSDATE, 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
        OR TO_CHAR (SYSDATE, 'DY') IN ('SAT', 'SUN') THEN
	RAISE_APPLICATION_ERROR (-20205,
		'You may only make changes during normal office hours');
  END IF;
END secure_dml;
```

The following list contains the error messages returned by the PostgreSQL server and manual changes to address those errors:
1. `ERROR: syntax error at or near "IS" - LINE 2: IS`: replace `IS` with `AS $$`
2. `ERROR: syntax error at or near "AS" - LINE 2: AS $$`: add missing `()` between `PROCEDURE secure_dml` and `AS $$`
3. `ERROR: unterminated dollar-quoted string at or near "$$ ... - LINE 2: AS $$` - replace `END secure_dml` with `END\n$$ LANGUAGE plpgsql`
4. `syntax error at or near "RAISE_APPLICATION_ERROR" - LINE 6: RAISE_APPLICATION_ERROR (-20205,`: replace procedure call to `RAISE_APPLICATION_ERROR` by `RAISE EXCEPTION 'You may only make changes during normal office hours' USING ERRCODE = '-20205';`

   After applying the changes listed above the `secure_dml` procedure can be created in PostgreSQL, but we still get an error when calling it.
    ```sql
    # call hr.secure_dml();
    ERROR:  column "sysdate" does not exist
    LINE 1: SELECT TO_CHAR (SYSDATE, 'HH24:MI') NOT BETWEEN '08:00' AND ...
                            ^
    QUERY:  SELECT TO_CHAR (SYSDATE, 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
            OR TO_CHAR (SYSDATE, 'DY') IN ('SAT', 'SUN')
    CONTEXT:  PL/pgSQL function hr.secure_dml() line 3 at IF
    ```

5. Replacing `SYSDATE` with `clock_timestamp()` leads to a functional equivalent on PostgreSQL with PL/pgSQL.
    ```sql
    -- Migrated procedure to PL/pgSQL
    CREATE PROCEDURE secure_dml()
    AS $$
    BEGIN
      IF TO_CHAR (clock_timestamp(), 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
            OR TO_CHAR (clock_timestamp(), 'DY') IN ('SAT', 'SUN') THEN
        RAISE EXCEPTION 'You may only make changes during normal office hours' USING ERRCODE = '-20205';
      END IF;
    END
    $$ LANGUAGE plpgsql;
    ```

### Migration Assessment
**Goal**: for each DBO (with code) assess the effort to migrate database code to PostgreSQL.

> **Note**  
> The ideal solution would provide a list of invalid PL/pgSQL constructs and
> * return one (or more) fix(es) which may be applied to the code
> * provide information if there is way to migrate the functionality (but not in the PL/pgSQL language space)
> * tell the user that there is no equivalent solution in PostgreSQL

Here a list of causes (not intended to be exhaustive) which may have impact on the migration effort:
* Function signature (number of arguments and its types, return type)
* Called functions/UDP (and how they are called)
* Code metrics
    * number of lines of code
    * number of statements
    * cyclomatic complexity (code path)
* Used SQL commands
    * DDL (Data Definition Language)
    * DML (Data Manipulation Language)
    * DCL (Data Control Language)
    * TCL (Transactional Control Language)
* Database specific functionality
    * external modules (`DMBS`)
    * `CONNECT BY`
    * `OUTER JOIN(+)`
    * `DECODE`
    * ...
* Unsupported language constructs
    * global variables in packages
    * anonymous/initialization block in package
    * nested functions

The metadata of our example `secure_dml`:
* signature: () -> ()
* called functions/procedures:
    * `RAISE_APPLICATION_ERROR`: 1
    * `SYSDATE`: 2
    * `TO_CHAR`: 2
* code metrics:
    * LOC: 9
    * statements: 2
    * cyclomatic complexity: 2

> **Note**  
> The user may choose to use [Orafce - Oracle's compatibility functions and packages](https://github.com/orafce/orafce) to reduce the migration effort.
> This has an impact on the assessment.

### Browser and WASM
The [Atroposs Database Migration Assessment (DMA) module](https://github.com/cybertec-postgresql/atroposs-dma-module) assesses the costs to migrate a database to PostgreSQL.

One of the requirements to the DMA module is to work in the browser, i.e. the database meta-data (DDL) never leaves the browser.

The user performs an assessment with the following steps:
1. Use the Oracle SQL Developer to export the database meta-data as a set of SQL files (with DDL commands) into a ZIP file.
2. On the Atroposs web-page go to the DMA module and start an assessment: select the ZIP file exported in the previous step.
3. The DMA module parses the minimal information from the ZIP file to create the database meta-data model (JavaScript SQL parser create with ANTLR4) 
4. For each DBO in the database meta-data model calculate the assessment to migrate the DBO to PostgreSQL
5. The GUI provides the means to interactively analyze the migration effort and its distribution

Step 4. does the heavy lifting since there we have to extract the meta-data for all PL/SQL code.
This means we have to interface the JavaScript (TypeScript) code with the analyzer written in Rust.

WebAssembly is a prime target of target with a large community and [first-hand endorse  ment](https://www.rust-lang.org/what/wasm). There also exists an [offical book](https://rustwasm.github.io/docs/book/).

### Transpiler to PL/pgSQL
If we can parse (analyze) the PL/SQL code we want to transpile code constructs to PL/pgSQL:
* automatically: for code constructs we know they can be transpiled correctly
* interactively ask for the user to apply one of the proposed fixes if the analyzer provides more than one solution

Show code constructs (as error/warning) which may not be transpiled because of a missing equivalent functionality on PostgreSQL.

## Desired Outcomes and Service Provision

Ferrous Systems provides its expertise with Rust, compiler construction and the tooling around lexers/parers to coach CYBERTEC's developers with the goal to find an optimal technical solution to the [problem field described above](#problem-description).

The team decides on concrete technologies and
* Implements a PoC written in Rust to extract the metadata of the Oracle UDP above returning the metadata
* Uses the functionality to calculate the assessment in the Atroposs DMA module (WASM,JavaScript)
* Uses the functionality to transpile the code automatically from PL/SQL to PL/pgSQL code.
* How the analyzer handles unknown language constructs
* How the analyzer handles language constructs with no equivalent on PostgreSQL

Describe solutions
* What work would we have to do if we add a new database technology (example: DB2) with slightly different syntax?
* How we solve the problem of conditional analysis with/without `Orafce`.
* How we can use the tooling to replace the ANTLR4 parser to create the database meta-data model.

> **Non-Goal**  
> Implementening a full fledged solution.

## Questions to Solve

By implementing the PoC we want to find answers/solutions to the following questions

* General software design: boundaries between the analyzer, assessment calculation, respective transpiler (apply fixes)
    * DDL(Oracle PL/SQL) --[analyzer]--> Fixes --[apply fixes]--> DDL(PL/pgSQL)
    * DDL(Oracle PL/SQL) --[analyzer]--> Fixes --[assessment]--> migration cost
    * --> How do we write a "Clippy for PL/pgSQL" which provides the fixes for the source PL/SQL code?

* Analyzer (and transpiler) are going to operate on abstract syntax trees (AST)
    * How could the data structure of a typed AST look like?
    * Do we have AST data types for each database technology (PostgreSQL, Oracle, DB2, ...) or do we have a super-set of AST with which we can represent all SQL dialects?
      - Christoph: I'd definitely go with the second option, an AST should be language-agnostic. Language-specific constructs can then be integrated into the AST as language-specific leafs (think: enum-like).
    * Perception of what we do:
        * We work only with PL/pgSQL AST - search for erroneous nodes  vs.
        * We start from an AST provided by the source PL/SQL code and apply transformations to map it to a PL/pgSQL AST
    * CYBERTEC Migrator: the code shows errors for invalid PL/pgSQL code
    * How can we connect the typed AST (Red Tree) with concrete realization (Green Tree from input source) so we can easily work with both?
    * Parametrized analysis: effort with/without Orafce?

* Lexer/parser: how are we going to create the trees?
    * Do we define a grammar and use a parser generator or should we write the parser manually using parser combinator frameworks?
      - Christoph: Definitely parser generator using grammar. Although parser combinator might be a bit more flexible (but not necessarily), the biggest contra is that they are slower. And since running it in a browser is a main goal, this goes directly against that. Parser combinators are usually used for small one-off parser, and thus really not that practible for parsing SQL. It would also result in a lot more code to maintain (and probably a bigger final compiled size, again might not be that suitable for use in browsers).
    * The goal is to reduce the maintenance effort and reduce the likelihood of errors.
        * How much work do we have to add a new database technology? A different version?
        * How much work do we have to for language constructs of a new database version?

* We don't want to reinvent the wheel: open source libraries we could use to base our work on?
    * There are SQL parser libraries can we use them as starting point?
      - [pest](https://pest.rs/): is probably one of the best-known and widely-used parser libraries. Also in active and regular development. (Christoph: I've used and worked with it before, would be me preferred choice.)
      - [lalrpop](https://github.com/lalrpop/lalrpop): Less popular/widely used, rather slow releases/development cycles.
      - [peg](https://github.com/kevinmehall/rust-peg): Smallest of all the options. Also uses [PEG](https://en.wikipedia.org/wiki/Parsing_expression_grammar) like pest.

* What is a goof iterative approach to show working code in a PoC?
  Start with the simplest constructs, enhance the analytical power in each iteration.
  We do not want to write a full fledged solution before we can extract the minimal information.

## References

An article I found recenetly regarding parsing SQL that might be of interest: (Parsing SQL - Strumenta)[https://tomassetti.me/parsing-sql/]
