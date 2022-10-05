-- test: parameter types blob
CREATE OR REPLACE PROCEDURE parameter_types_blob(
    p_blob bytea
    , p_binary_large_object byeta
)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
