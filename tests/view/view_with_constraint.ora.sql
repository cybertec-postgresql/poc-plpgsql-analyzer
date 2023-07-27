CREATE VIEW view_with_constraint (
  store_id,
  name,
  email UNIQUE RELY DISABLE NOVALIDATE,
  CONSTRAINT store_id_pk PRIMARY KEY (store_id) RELY DISABLE NOVALIDATE)
AS SELECT store_id, name, email FROM stores;
