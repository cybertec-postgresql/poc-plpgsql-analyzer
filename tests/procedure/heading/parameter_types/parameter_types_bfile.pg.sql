-- test: parameter types bfile
CREATE OR REPLACE PROCEDURE parameter_types_bfile(p_bfile bytea)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
