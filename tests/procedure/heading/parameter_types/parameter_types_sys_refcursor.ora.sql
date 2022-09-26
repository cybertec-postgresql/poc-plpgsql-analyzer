-- test: frequently used parameter types
CREATE OR REPLACE PROCEDURE parameter_types_sys_refcursor(
    p_sys_refcurosor IN OUT NOCOPY sys_refcursor
)
IS
BEGIN
    NULL;
END parameter_types_sys_refcursor;
