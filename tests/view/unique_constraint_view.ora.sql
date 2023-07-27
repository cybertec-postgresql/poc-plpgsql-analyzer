CREATE VIEW unique_constraint_view (
  store_id,
  email UNIQUE RELY DISABLE NOVALIDATE,
  CONSTRAINT store_pk PRIMARY KEY (store_id) RELY DISABLE NOVALIDATE)
AS SELECT store, email FROM stores;
