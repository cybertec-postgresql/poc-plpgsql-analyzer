-- test: OUT parameter
CREATE OR REPLACE PROCEDURE parameter_modes_out(
    -- no support for OUT paramaters for PROCEDURE prior of PostgreSQL 14 (use INOUT)
    foo INOUT text
)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
