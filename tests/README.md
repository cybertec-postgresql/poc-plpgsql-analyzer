# Test Cases
As PL/SQL reference we pick [Oracle 12c PL/SQL Language Reference](https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/index.html) which end of support is in 2022 and thus most probable to switch to PostgreSQL.

## Table of Contents
* [Function](#function)
* [Procedure](#procedure)
    * [Procedure Heading](#procedure-heading)
    * [Declare Section](#declare-section)
    * [Procedure Body](#procedure-body)
* [Triggers](#triggers)

## References
* [SQL Statements for Stored PL/SQL Units](https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/sql-statements-for-stored-plsql-units.html#GUID-C918310F-F1BB-41D7-9466-B558B70DDFFE)
* [Oracle - Database PL/SQL Language Reference - PL/SQL Data Types](https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/plsql-data-types.html#GUID-391C58FD-16AF-486C-AF28-173E309CDBA5)
* [Oracle Documentation - PL/SQL Predefined Data Types](https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/plsql-predefined-data-types.html#GUID-1D28B7B6-15AE-454A-8134-F8724551AE8B)
* [PostgreSQL Documentation - PL/pgSQL](https://www.postgresql.org/docs/current/plpgsql.html)

## Function
References
- [Oracle - Database PL/SQL Language Reference - Function Declaration and Definition](https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/function-declaration-and-definition.html#GUID-4E19FB09-46B5-4CE5-8A5B-CD815C29DA1C)
- [Oracle Documentation - CREATE FUNCTION statement](https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/CREATE-FUNCTION-statement.html#GUID-B71BC5BD-B87C-4054-AAA5-213E856651F2)

### Function Heading
- See the [Procedure](#procedure).
- `DETERMINISTIC` keyword

## Procedure
References
- [Oracle - Database PL/SQL Language Reference - Procedure Declaration and Definition](https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/procedure-declaration-and-definition.html#GUID-9A48D7CE-3720-46A4-B5CA-C2250CA86AF2)
- [Oracle Documentation - CREATE PROCEDURE statement]( https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/CREATE-PROCEDURE-statement.html#GUID-5F84DB47-B5BE-4292-848F-756BF365EC54)

### Procedure Heading
Minimal functionality:
* Replace `IS ... BEGIN ... END name;`  with `AS $$ ... BEGIN ... END; $$ LANGUAGE plpgsql;`
* [Parameter declaration](https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/formal-parameter-declaration.html#GUID-5BA8E033-96B9-439A-A4FC-4844FEC14AD8)
    - convert data types
    - convert `:=` to `DEFAULT`
    - argument modifier `IN OUT` to `INOUT`
    - replace `OUT` with `INOUT` (support for `OUT` in procedures was added with PostgreSQL 14)
    - ignore `NOCOPY` keyword
    - convert Oracle specific default expressions (`SYSDATE`, ...)
    - fix `IN` parameter after one with default value must also have defaults

#### Example
* Oracle
    ```sql
    -- test: example for procedure heading
    CREATE OR REPLACE PROCEDURE procedure_heading_example (
        -- no support for OUT parameters for PROCEDURE prior of PostgreSQL 14 (use INOUT)
        p_1 OUT VARCHAR2
        , p_2 NUMBER
        , p_3 IN BOOLEAN := FALSE
        -- IN parameter after one with default value must also have defaults
        , p_4 IN OUT NOCOPY DATE
        , p_5 foo.bar%TYPE
    )
    IS
    BEGIN
        NULL;
    END procedure_heading_example;
    ```
* PostgreSQL
    ```sql
    -- test: example for procedure heading
    CREATE OR REPLACE PROCEDURE procedure_heading_example (
        -- no support for OUT parameters for PROCEDURE prior of PostgreSQL 14 (use INOUT)
        p_1 INOUT text
        , p_2 bigint
        , p_3 boolean DEFAULT FALSE
        -- IN parameter after one with default value must also have defaults
        , p_4 INOUT timestamp DEFAULT NULL
        , p_5 foo.bar%TYPE DEFAULT NULL
    )
    AS $body$
    BEGIN
        NULL;
    END;
    $body$ LANGUAGE plpgsql;
    ```

### Declare Section
Minimal functionality:
* Add `DECLARE` keyword if missing
* Convert data types
* Convert data types for `TYPE identifier IS TABLE OF`
* `CURSOR identifier IS SELECT ...` to `identifier CURSOR FOR SELECT ...`

TODO:
* Convert cursors (TODO)
* Don't migrate exceptions

### Procedure Body
Minimal functionality:
* add `CALL` statement for procedure invocation (for procedures without output parameters)
* exception handling
    - `SQLCODE` vs. `SQLSTATE`
    - `RAISE EXCEPTION`
    - cursor `EXIT WHEN curs1%NOTFOUND` with `EXIT WHEN NOT FOUND`
* array index: `A(i)` to `A[i]`
* `SELECT ... FROM DUAL`
* Convert `SELECT ... INTO ... FROM` into `SELECT ... INTO STRICT ... FROM`

## Unknown Functions
`ora2pg` addresses following incompatible/unknown functions:
* `SYSDATE` (`ora2pg` sometimes uses `clock_timestamp` sometimes `LOCALTIMESTAMP`)
* `RAISE_APPLICATION_ERROR`
* `NVL`
    * bonus: replace nested `NVL` with one call to `COALESCE`
* `REGEXP_LIKE()` to `regexp_match`
* `LISTAGG(...) WITHIN GROUP (ORDER BY ...)` to `STRING_AGG|ARRAY_AGG`
* `TO_NUMBER(n)` to `(n)::numeric`
* `TRUNC(date)` with `date_trunc('day', date)`
* Oracle modules:
    * replace `SYS.DBMS_OUTPUT.PUT_LINE()` with `RAISE NOTICE`, replace `||` concatenation with `% %`

## Triggers
* covert `:[old|new]` to `OLD` respective `NEW`

## SELECT Statement
* Oracle `OUTER JOIN(+)` syntax
* `WHERE ROWNUM` to `LIMIT`
* `CONNECT BY`

## ora2pg Features
* Use of `dblink` to implement `PRAGMA AUTONOMOUS_TRANSACTION` (see )

## Convert Code with ora2pg
* Use a container image from `docker.io/georgmoser/ora2pg` to skip a complicate installation process
  > **Note**  
  > Since the container image doesn't provide a default configuration at `/etc/ora2pg/ora2pg.conf` we have to provide an empty configuration file.

  ```sh
  # Convert code in `procedure/heading/parameter_declaration_empty.ora.sql` to `parameter_declaration_empty.pg.sql`
  export TYPE=procedure
  export DIR=$TYPE/heading
  export TEST_CASE=parameter_declartion_empty
  touch $DIR/empty.conf
  docker run --rm -it -u $UID -v "$(pwd)/$DIR":"/$DIR" -w "/$DIR" georgmoser/ora2pg ora2pg -c empty.conf -t $TYPE -i $TEST_CASE.ora.sql -o $TEST_CASE.pg.sql
  ```
