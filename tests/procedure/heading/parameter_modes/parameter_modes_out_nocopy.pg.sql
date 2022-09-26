-- test: OUT NOCOPY parameter
CREATE OR REPLACE PROCEDURE parameter_modes_out_nocopy(
    -- no support for OUT parameter for PROCEDURE prior of PostgreSQL 14 (use INOUT)
    foo INOUT text
)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
