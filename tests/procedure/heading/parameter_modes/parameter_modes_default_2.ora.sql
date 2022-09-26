-- test: IN parameter with DEFAULT value
CREATE OR REPLACE PROCEDURE parameter_modes_default_2(foo IN VARCHAR2 DEFAULT 'not empty')
IS
BEGIN
    NULL;
END parameter_modes_default_2;
