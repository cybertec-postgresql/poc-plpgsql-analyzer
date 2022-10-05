# Test Cases

As PL/SQL reference we pick [Oracle 12c PL/SQL Language Reference](https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/index.html) which end of support is in 2022 and thus most probable to switch to PostgreSQL.

## Table of Contents

* [Procedure](#procedure)
    * [Procedure Heading](#procedure-heading)
    * [Declare Section](#declare-section)
    * [Procedure Body](#procedure-body)

## Procedure

References
- [Oracle Documentation - CREATE PROCEDURE statement]( https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/CREATE-PROCEDURE-statement.html#GUID-5F84DB47-B5BE-4292-848F-756BF365EC54)
- [Oracle Documentation - PL/SQL Predefined Data Types](https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/plsql-predefined-data-types.html#GUID-1D28B7B6-15AE-454A-8134-F8724551AE8B)

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
        -- no support for OUT paramaters for PROCEDURE prior of PostgreSQL 14 (use INOUT)
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
        -- no support for OUT paramaters for PROCEDURE prior of PostgreSQL 14 (use INOUT)
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

TODO

### Procedure Body

TODO

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

## References

* [SQL Statements for Stored PL/SQL Units](https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/sql-statements-for-stored-plsql-units.html#GUID-C918310F-F1BB-41D7-9466-B558B70DDFFE)
* [PostgreSQL Documentation - PL/pgSQL](https://www.postgresql.org/docs/current/plpgsql.html)

