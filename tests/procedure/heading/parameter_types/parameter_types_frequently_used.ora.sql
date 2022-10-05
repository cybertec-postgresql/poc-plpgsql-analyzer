-- test: frequently used parameter types
CREATE OR REPLACE PROCEDURE parameter_types_frequently_used(
    p_blob BLOB
    , p_boolean BOOLEAN
    , p_char CHAR
    , p_cblob CLOB
    , p_date DATE
    , p_integer INTEGER
    , p_number NUMBER
    , p_pls_integer PLS_INTEGER
    , p_varchar VARCHAR
    , p_varchar2 VARCHAR2
    , foo_bar foo.bar%TYPE
)
IS
BEGIN
    NULL;
END parameter_types_frequently_used;
