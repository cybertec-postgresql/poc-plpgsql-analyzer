-- test: parameter types CLOBs
-- TODO: expected values
CREATE OR REPLACE PROCEDURE parameter_types_clob(
    p_clob text
    , p_character_large_object CHARACTER LARGE OBJECT
    , p_char_large_object CHAR LARGE OBJECT
    , p_national_large_bject NATIONAL CHARACTER LARGE OBJECT
    , p_nchar_large_bject NCHAR LARGE OBJECT
    , p_nclob text
)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
