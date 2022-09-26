-- test: simple parameter without IN keyword
CREATE OR REPLACE PROCEDURE parameter_modes_none(foo text)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
