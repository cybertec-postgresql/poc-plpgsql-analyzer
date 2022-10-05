-- test: IN parameter with default value using =:
CREATE OR REPLACE PROCEDURE parameter_modes_default_4(foo IN VARCHAR2 := 'not empty')
IS
BEGIN
    NULL;
END parameter_modes_default_4;
