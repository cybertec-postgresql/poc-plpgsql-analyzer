-- test: multiple parameters IN and OUT
CREATE OR REPLACE PROCEDURE multiple_parameters_in_out(
    p1 text
    -- no support for OUT paramaters for PROCEDURE prior of PostgreSQL 14 (use INOUT)
     , p2 INOUT text
)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
