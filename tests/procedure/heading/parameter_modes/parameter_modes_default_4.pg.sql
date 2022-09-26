-- test: IN parameter with default value using =:
CREATE OR REPLACE PROCEDURE parameter_modes_default_4(foo IN text DEFAULT 'not empty')
AS $body$
BEGIN
    NULL;
END;
$body$ LANGUAGE plpgsql;
