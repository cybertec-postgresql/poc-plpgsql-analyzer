-- test: IN OUT NOCOPY parameter
CREATE OR REPLACE PROCEDURE parameter_modes_in_out_nocopy(foo IN OUT NOCOPY VARCHAR2)
IS
BEGIN
    NULL;
END parameter_modes_in_out_nocopy;
