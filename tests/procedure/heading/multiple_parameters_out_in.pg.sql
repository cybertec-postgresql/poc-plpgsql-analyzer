-- test: multiple parameters OUT before IN
CREATE OR REPLACE PROCEDURE multiple_parameters_out_in(
    -- no support for OUT paramaters for PROCEDURE prior of PostgreSQL 14 (use INOUT)
    p1 INOUT text
    , p2 text
)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
