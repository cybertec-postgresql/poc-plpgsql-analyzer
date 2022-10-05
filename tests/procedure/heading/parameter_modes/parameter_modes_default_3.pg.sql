-- test: parameter with default value using =:
CREATE OR REPLACE PROCEDURE parameter_modes_default_3(foo text DEFAULT 'not empty')
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
