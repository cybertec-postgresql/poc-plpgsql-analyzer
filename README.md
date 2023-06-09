# Pl/pgSQL Analyzer

[![Rust](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/technology-WebAssembly-blue.svg)](https://webassembly.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

Proof of concept tooling library to migrate PL/SQL code to PL/pgSQL, written in Rust.

A Rust WebAssembly (WASM) application that parses Oracle PL/SQL code, assesses its structure, and transpiles it into
equivalent PostgreSQL PL/pgSQL code.

## Features

- PL/SQL Parser: Parses Oracle PL/SQL code to build an appropriate AST.
- Code Assessment: Analyzes the structure and complexity of migrating the PL/SQL code.
- Transpiler: Converts Oracles PL/SQL code to PostgreSQLs PL/pgSQL.
- WebAssembly: Utilizes Rust's WebAssembly support for running in the browser.

## Problem Description

Given a valid SQL DDL command to `CREATE` a database object (DBO), this library should parse the statement, extract its
metadata, and calculate a quantitative measure (expressed in an abstract unit) representing the effort required to
migrate the DBO to PostgreSQL.

DBOs that typically demand significant effort for migration to PostgreSQL include:

- Functions
- Packages
- Procedures
- Triggers
- Views

In addition to the aforementioned DBOs, it may be necessary to evaluate the effort involved in migrating other language
constructs to PostgreSQL, such as:

- Column check constraint expressions
- Column default expressions
- Expressions in functional indexes

The error handling mechanism should exhibit leniency. In instances where the analyzer fails to parse a particular
language construct, it should provide a descriptive error output and promptly attempt to continue the analysis of the
remaining components.

### Example

To showcase the necessary modifications a user has to undertake during migration, we will utilize the `SECURE_DML`
procedure of Oracles `HR` sample schema.

```sql
-- Oracle PL/SQL
CREATE PROCEDURE secure_dml
    IS
BEGIN
  IF
TO_CHAR (SYSDATE, 'HH24:MI') NOT BETWEEN '08:00' AND '18:00'
        OR TO_CHAR (SYSDATE, 'DY') IN ('SAT', 'SUN') THEN
      RAISE_APPLICATION_ERROR (-20205, 'You may only make changes during normal office hours');
END IF;
END secure_dml;
```

Listed below are the error messages emitted by the PosgreSQL server, accompanied by the corresponding manual changes
required to resolve said errors:

1. `ERROR: syntax error at or near "IS" - LINE 2: IS`:\
   Replace `IS` with `AS $$`
2. `ERROR: syntax error at or near "AS" - LINE 2: AS $$`:\
   Insert the missing `()` between `PROCEDURE secure_dml` and `AS $$`
3. `ERROR: unterminated dollar-quoted string at or near "$$ ... - LINE 2: AS $$`:\
   Replace `END secure_dml` with `END\n$$ LANGUAGE plpgsql`
4. `syntax error at or near "RAISE_APPLICATION_ERROR" - LINE 6: RAISE_APPLICATION_ERROR (-20205,`:\
   Replace procedure call `RAISE_APPLICATION_ERROR`
   with `RAISE EXCEPTION 'You may only make changes during normal office hours' USING ERRCODE = '-20205';`

   \
   Upon implementing the aforementioned modifications, the `secure_dml` procedure can be created PostgreSQL. However, we
   will encounter an error when invoking the procedure:

    ```sql
    postgres=# CALL hr.secure_dml();
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

**Objective**: Evaluate the effort required to migrate the DBOs along with its corresponding code to PostgreSQL.

Using `secure_dml` as an example, the metadata would be:

- Signature: `() -> ()`
- Function/procedure invocations:
    - `RAISE_APPLICATION_ERROR: 1`
    - `SYSDATE: 2`
    - `TO_CHAR: 2`
- Code metrics:
    - `Lines of code: 9`
    - `Number of statements: 2`
    - `Cyclomatic complexity: 2`

> **Note**  
> The user may choose to use [Orafce - Oracle's compatibility functions and packages](https://github.com/orafce/orafce)
> to reduce the migration effort. This choice will have an impact on the assessment figures.

## Coverage

### Parser coverage

Given a valid `CREATE` SQL DDL command:

| Construct               | Supported |   
|-------------------------|-----------|
| Constraints             | ❌         |
| DB Links                | ❌         |
| Functions               | ✅         |
| Indexes                 | ❌         |
| Operators               | ❌         |
| Packages                | ❌         |
| Procedures              | ✅         |
| Queues                  | ❌         |
| Referential constraints | ❌         |
| Sequences               | ❌         |
| Tables                  | ❌         |
| Triggers                | ✅         |
| User-defined types      | ❌         |
| Views                   | ✅         |

#### Oracle statements coverage

Most Oracle code in one way or another makes use of
the [`BLOCK`](https://docs.oracle.com/en/database/oracle/oracle-database/21/lnpls/block.html#GUID-9ACEB9ED-567E-4E1A-A16A-B8B35214FC9D),
allowing multiple statements within.

| Statement                   | Supported |   
|-----------------------------|-----------|
| Assignment statement        | ❌         |
| Basic loop statement        | ❌         |
| Case statement              | ❌         |
| Close statement             | ❌         |
| Collection method call      | ❌         |
| Continue statement          | ❌         |
| Cursor for loop statement   | ❌         |
| Declare section             | ❌         |
| Execute immediate statement | ❌         |
| Exit statement              | ❌         |
| Fetch statement             | ❌         |
| For loop statement          | ❌         |
| Forall statement            | ❌         |
| Goto statement              | ❌         |
| If statement                | ✅         |
| Null statement              | ✅         |
| Open for statement          | ❌         |
| Open statement              | ❌         |
| Nested PL/SQL block         | ✅         |
| Pipe row statement          | ❌         |
| Procedure call              | ✅         |
| Raise statement             | ❌         |
| Return statement            | ✅         |
| SQL statement               | Partially |
| Select into statement       | ✅         |
| While loop statement        | ❌         |

##### Declare section

| Statement                       | Supported |   
|---------------------------------|-----------|
| Collection type definition      | ❌         |
| Collection variable declaration | ❌         |
| Constant declaration            | ❌         |
| Cursor declaration              | ❌         |
| Cursor variable declaration     | ❌         |
| Exception declaration           | ❌         |
| Function declaration            | ❌         |
| Procedure declaration           | ❌         |
| Record type definition          | ❌         |
| Record variable declaration     | ❌         |
| Ref cursor type definition      | ❌         |
| Subtype definition              | ❌         |
| Variable declaration            | ❌         |

##### SQL Statements

| Statement                 | Supported |   
|---------------------------|-----------|
| Collection method call    | ❌         |
| Commit statement          | ❌         |
| Delete statement          | ❌         |
| Insert statement          | ✅         |
| Lock table statement      | ❌         |
| Merge statement           | ❌         |
| Rollback statement        | ❌         |
| Savepoint statement       | ❌         |
| Set transaction statement | ❌         |
| Update statement          | ❌         |

### Analyzer coverage

#### Supported

- Code metrics
    - Lines of code
- Database specific functionality
    - Outer joins using the `(+)` syntax

#### To be implemented

- Function signature
- Called functions/procedures
- Code metrics
    - Number of statements
    - Cyclomatic complexity (code path)
- Used SQL commands
    - DDL (Data Definition Language)
    - DML (Data Manipulation Language)
    - DCL (Data Control Language)
    - TCL (Transactional Control Language)
- Database specific functionality
    - External modules (e.g. `DBMS`)
    - `CONNECT BY`
    - `DECODE`
    - and many more.
- Unsupported language constructs
    - Global variables in packages
    - Anonymous/initialization block in package
    - Nested functions

## Contributing

Refer to the [`Development README`](./DEVELOPMENT.md).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## References

- [Extensible SQL Lexer and Parser for Rust](https://github.com/sqlparser-rs/sqlparser-rs)
- [Blog article: Parsing SQL - Strumenta](https://tomassetti.me/parsing-sql/)
- [Blog article: Easy Lossless Trees with Nom and Rowan](https://blog.kiranshila.com/blog/easy_cst.md)
- [Blog article: Introducing Rust Sitter](https://www.shadaj.me/writing/introducing-rust-sitter/)
