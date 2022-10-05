-- test: parameter types NUMBER
CREATE OR REPLACE PROCEDURE parameter_types_number(
    p_number NUMBER
    , p_float FLOAT
    , p_real REAL
    , p_double_precision DOUBLE PRECISION
    , p_integer INTEGER
    , p_int INT
    , p_smallint SMALLINT
    , p_decimal DECIMAL
    , p_numeric NUMERIC
    , p_dec DEC
    , p_binary_integer BINARY_INTEGER
    , p_natural NATURAL
    , p_naturaln NATURALN
    , p_positive POSITIVE
    , p_positiven POSITIVEN
    , p_signtype SIGNTYPE
    , p_pls_integer PLS_INTEGER
    , p_binary_float BINARY_FLOAT
    , p_binary_double BINARY_DOUBLE
    , p_simple_integer SIMPLE_INTEGER
    , p_simple_float SIMPLE_FLOAT
    , p_simple_double SIMPLE_DOUBLE
)
IS
BEGIN
    NULL;
END parameter_types_number;
/
