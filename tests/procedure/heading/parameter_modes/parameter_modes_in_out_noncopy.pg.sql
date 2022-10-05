-- test: IN OUT NOCOPY parameter
CREATE OR REPLACE PROCEDURE parameter_modes_in_out_nocopy(foo INOUT text)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
