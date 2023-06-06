CREATE OR REPLACE EDITIONABLE TRIGGER store.after_trigger
  AFTER UPDATE OF order_id, group_id ON customers
  FOR EACH ROW
BEGIN
  add_history(:old.customer_id, :old.created_at, sysdate,
              :old.order_id, :old.group_id);
END;
