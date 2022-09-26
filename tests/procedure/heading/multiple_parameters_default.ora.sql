-- test: multiple parameters with default values
CREATE OR REPLACE PROCEDURE multiple_parameters_default(
    p1 VARCHAR2
    , p2 VARCHAR2 := 'not empty'
)
IS
BEGIN
    NULL;
END multiple_parameters_default;
