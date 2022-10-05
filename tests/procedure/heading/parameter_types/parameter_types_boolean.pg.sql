-- test: parameter types boolean
-- ref: https://docs.oracle.com/en/database/oracle/oracle-database/12.2/lnpls/plsql-predefined-data-types.html#GUID-1D28B7B6-15AE-454A-8134-F8724551AE8B
CREATE OR REPLACE PROCEDURE parameter_types_boolean (p1 boolean)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
