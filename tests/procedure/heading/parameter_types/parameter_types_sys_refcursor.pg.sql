-- test: frequently used parameter types
CREATE OR REPLACE PROCEDURE parameter_types_sys_refcursor(
    p_sys_refcurosor INOUT REFCURSOR
)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
