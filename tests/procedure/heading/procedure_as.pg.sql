-- test: use of AS instead of IS
CREATE OR REPLACE PROCEDURE procedure_as ()
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
