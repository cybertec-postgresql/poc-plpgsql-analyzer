-- test: multiple parameters IN and OUT
CREATE OR REPLACE PROCEDURE multiple_parameters_in_out(
    p1 IN VARCHAR2
    -- no support for OUT paramaters for PROCEDURE prior of PostgreSQL 14 (use INOUT)
    , p2 OUT VARCHAR2
)
IS
BEGIN
    NULL;
END multiple_parameters_in_out;
