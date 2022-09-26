-- test: IN OUT parameter
CREATE OR REPLACE PROCEDURE parameter_modes_in_out(foo INOUT text)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
