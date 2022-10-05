-- test: IN parameter
CREATE OR REPLACE PROCEDURE parameter_modes_in(foo IN text)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
