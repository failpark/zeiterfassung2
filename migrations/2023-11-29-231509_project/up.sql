CREATE TABLE project (
	id INTEGER AUTO_INCREMENT NOT NULL,
	client_id INTEGER NOT NULL,
	name VARCHAR(255) NOT NULL,
	created_at TIMESTAMP NOT NULL
		DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL
		DEFAULT CURRENT_TIMESTAMP
		ON UPDATE CURRENT_TIMESTAMP,
	PRIMARY KEY (id),
	FOREIGN KEY (client_id)
		REFERENCES client(id)
);