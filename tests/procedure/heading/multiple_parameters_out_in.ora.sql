-- test: multiple parameters OUT before IN
CREATE OR REPLACE PROCEDURE multiple_parameters_out_in(
    -- no support for OUT paramaters for PROCEDURE prior of PostgreSQL 14 (use INOUT)
    p1 OUT VARCHAR2
    , p2 IN VARCHAR2
)
IS
BEGIN
    NULL;
END multiple_parameters_out_in;
