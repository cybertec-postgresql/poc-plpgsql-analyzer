-- test: parameter with default value using :=
CREATE OR REPLACE PROCEDURE parameter_modes_default_3(foo VARCHAR2 := 'not empty')
IS
BEGIN
    NULL;
END parameter_modes_default_3;
