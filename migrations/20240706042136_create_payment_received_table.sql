-- Add migration script here

-- the first part of the payment
-- after it is confirmed there is a delay for it being received(boleto alguns dias),credito 1 mes
-- importante ter esses dados para conseguir controlar os refunds
create table payment_confirmed (
      id SERIAL PRIMARY KEY,
      event_id VARCHAR(255) NOT NULL,
      cliente_id INT NOT NULL,
      --its added later, to confirm the payment
      payment_date DATE NOT NULL,
      created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE payment_received (
   id SERIAL PRIMARY KEY,
   -- "evt_05b708f961d739ea7eba7e4db318f621&368604920"
   event_id VARCHAR(255) NOT NULL,
   payment_date DATE NOT NULL,
   payment_confirmed INT NOT NULL,
   created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
-- TODO there is no oon update, will need to have a trigger or do it on the code 
--   updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, ON UPDATE CURRENT_TIMESTAMP,
-- deleted_at TIMESTAMP DEFAULT NULL,
--    created_by INT NOT NULL,
--   updated_by INT NOT NULL,
--    deleted_by INT DEFAULT NULL,
   FOREIGN KEY (payment_confirmed) REFERENCES payment_confirmed(id)
--   FOREIGN KEY (created_by) REFERENCES user(id),
--  FOREIGN KEY (updated_by) REFERENCES user(id),
-- FOREIGN KEY (deleted_by) REFERENCES user(id)
);

--use this to check credit card
--the refund is deleted when the payment is made
CREATE table payment_refunded (
   id SERIAL PRIMARY KEY,
   event_id VARCHAR(255) NOT NULL,
   payment_received_id INT NOT NULL,
   refund_date DATE NOT NULL,
   created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
   FOREIGN KEY (payment_received_id) REFERENCES payment_received(id)
);