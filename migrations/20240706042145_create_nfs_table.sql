-- Add migration script here
--will have an id,payment_received_id,path for the saved file,created_at,updated_at and sent(bool)
CREATE TABLE nfs (
    id INT PRIMARY KEY AUTO_INCREMENT,
    payment_received_id INT NOT NULL,
    path VARCHAR(255) NOT NULL,
    sent BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (payment_received_id) REFERENCES payment_received(id)
);