CREATE TABLE user ( 
	id INTEGER AUTO_INCREMENT NOT NULL,
	username VARCHAR(50) NOT NULL,
	firstname VARCHAR(40) NOT NULL,
	lastname VARCHAR(40) NOT NULL,
	email VARCHAR(255) NOT NULL,
	hash VARCHAR(255) NOT NULL,
	sys_role VARCHAR(255) NOT NULL
		DEFAULT 'user',
	created_at TIMESTAMP NOT NULL
		DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL
		DEFAULT CURRENT_TIMESTAMP
		ON UPDATE CURRENT_TIMESTAMP,
	UNIQUE (username),
	PRIMARY KEY (id)
);