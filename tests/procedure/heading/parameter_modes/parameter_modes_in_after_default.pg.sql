-- test: IN parameter after one with a default value
CREATE OR REPLACE PROCEDURE parameter_modes_in_after_default(
    p1 IN text DEFAULT 'not empty'
    -- TODO: check what the behavior in Oracle is
    , p2 IN text DEFAULT NULL
)
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
