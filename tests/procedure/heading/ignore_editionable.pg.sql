-- test: ignore EDITIONABLE keyword, there is no equivalent in PostgreSQL
CREATE OR REPLACE PROCEDURE ignore_editionable()
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
