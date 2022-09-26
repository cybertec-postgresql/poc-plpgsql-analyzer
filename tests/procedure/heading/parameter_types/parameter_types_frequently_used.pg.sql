-- test: frequently used parameter types
CREATE OR REPLACE PROCEDURE parameter_types_frequently_used(
    p_blob bytea
    , p_boolean boolean
    , p_char CHAR
    , p_cblob text
    , p_date timestamp
    , p_integer integer
    , p_number bigint
    , p_pls_integer integer
    , p_varchar text
    , p_varchar2 text
    , foo_bar foo.bar%TYPE
) AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
