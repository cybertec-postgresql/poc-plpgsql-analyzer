CREATE OR REPLACE TRIGGER instead_of_trigger
    INSTEAD OF INSERT ON my_view
    FOR EACH ROW
DECLARE
    id NUMBER;
BEGIN
    NULL;
    -- insert a new customer first
    INSERT INTO customers(name, address, website, credit_limit)
    VALUES(:NEW.NAME, :NEW.address, :NEW.website, :NEW.credit_limit)
    RETURNING customer_id INTO id;

    -- insert the contact
    INSERT INTO contacts(first_name, last_name, email, phone, customer_id)
    VALUES(:NEW.first_name, :NEW.last_name, :NEW.email, :NEW.phone, id);
END;

