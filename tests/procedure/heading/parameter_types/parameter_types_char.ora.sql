-- test: parameter types char familiy
CREATE OR REPLACE PROCEDURE parameter_types_char(
    p_varchar2 VARCHAR2
    , p_varchar VARCHAR
    , p_urowid UROWID
    , p_string STRING
    , p_long LONG
    , p_raw RAW
    , p_long_raw LONG RAW
    , p_rowid ROWID
    , p_char CHAR
    , p_character_varying CHARACTER VARYING
    , p_char_varying CHAR VARYING
    , p_national_character NATIONAL CHARACTER
    , p_national_char NATIONAL CHAR
    , p_nchar NCHAR
    , p_nchar2 NVARCHAR2
)
IS
BEGIN
    NULL;
END parameter_types_char;
