-- test: IN parameter after one with a default value
CREATE OR REPLACE PROCEDURE parameter_modes_in_after_default(
    p1 IN VARCHAR2 := 'not empty'
    -- TODO: check what the behavior in Oracle is
    , p2 IN VARCHAR2
)
IS
BEGIN
    NULL;
END parameter_modes_in_after_default;
