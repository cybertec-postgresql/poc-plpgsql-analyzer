-- test: parameter with DEFAULT value
CREATE OR REPLACE PROCEDURE parameter_modes_default_1(foo text DEFAULT 'not empty')
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
