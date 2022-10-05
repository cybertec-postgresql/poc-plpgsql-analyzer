-- test: parameter types NUMBER
CREATE OR REPLACE PROCEDURE parameter_types_number (
    p_number bigint
    , p_float double precision
    , p_real real
    , p_double_precision double precision
    , p_integer integer
    , p_int integer
    , p_smallint smallint
    , p_decimal DECIMAL
    , p_numeric NUMERIC
    , p_dec DEC
    , p_binary_integer integer
    , p_natural NATURAL
    , p_naturaln NATURALN
    , p_positive POSITIVE
    , p_positiven POSITIVEN
    , p_signtype SIGNTYPE
    , p_pls_integer integer
    , p_binary_float numeric
    , p_binary_double numeric
    , p_simple_integer SIMPLE_INTEGER
    , p_simple_float SIMPLE_FLOAT
    , p_simple_double SIMPLE_DOUBLE
)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
