-- test: parameter types CLOBs
CREATE OR REPLACE PROCEDURE parameter_types_clob(
    p_clob CLOB
    , p_character_large_object CHARACTER LARGE OBJECT
    , p_char_large_object CHAR LARGE OBJECT
    , p_national_large_bject NATIONAL CHARACTER LARGE OBJECT
    , p_nchar_large_bject NCHAR LARGE OBJECT
    , p_nclob NCLOB
)
IS
BEGIN
    NULL;
END parameter_types_clob;
