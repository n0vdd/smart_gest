-- Add migration script here
-- will have an id, event_id,will link to cliente_id and the payment_date
CREATE TABLE payment_received (
    id INT PRIMARY KEY AUTO_INCREMENT,
    --i dont think varchar 255 is enough for the event_id
    -- "evt_05b708f961d739ea7eba7e4db318f621&368604920"
    event_id VARCHAR(255) NOT NULL,
    cliente_id INT NOT NULL,
    payment_date DATE NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
   -- deleted_at TIMESTAMP DEFAULT NULL,
--    created_by INT NOT NULL,
 --   updated_by INT NOT NULL,
--    deleted_by INT DEFAULT NULL,
    FOREIGN KEY (cliente_id) REFERENCES cliente(id)
 --   FOREIGN KEY (created_by) REFERENCES user(id),
  --  FOREIGN KEY (updated_by) REFERENCES user(id),
   -- FOREIGN KEY (deleted_by) REFERENCES user(id)
);