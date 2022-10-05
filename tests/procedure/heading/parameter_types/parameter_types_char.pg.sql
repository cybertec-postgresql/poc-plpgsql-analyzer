-- test: parameter types char familiy
-- TODO: expected values
CREATE OR REPLACE PROCEDURE parameter_types_char(
    p_varchar2 text
    , p_varchar text
    , p_urowid oid
    , p_string text
    , p_long text
    , p_raw bytea
    , p_long_raw bytea
    , p_rowid oid
    , p_char CHAR
    , p_character_varying CHARACTER VARYING
    , p_char_varying CHAR VARYING
    , p_national_character NATIONAL CHARACTER
    , p_national_char NATIONAL CHAR
    , p_nchar NCHAR
    , p_nchar2 NVARCHAR2
)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
