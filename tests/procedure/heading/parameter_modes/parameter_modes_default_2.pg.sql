-- test: IN parameter with DEFAULT value
CREATE OR REPLACE PROCEDURE parameter_modes_default_2(foo IN text DEFAULT 'not empty')
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
