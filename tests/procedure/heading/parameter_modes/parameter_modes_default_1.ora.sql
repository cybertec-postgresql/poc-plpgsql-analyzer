-- test: parameter with DEFAULT value
CREATE OR REPLACE PROCEDURE parameter_modes_default_1(foo VARCHAR2 DEFAULT 'not empty')
IS
BEGIN
    NULL;
END parameter_modes_default_1;
