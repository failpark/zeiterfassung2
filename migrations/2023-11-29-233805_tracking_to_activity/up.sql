CREATE TABLE tracking_to_activity (
	id INTEGER AUTO_INCREMENT NOT NULL,
	tracking_id INTEGER NOT NULL,
	activity_id INTEGER NOT NULL,
	PRIMARY KEY (id),
	FOREIGN KEY (tracking_id)
		REFERENCES tracking(id),
	FOREIGN KEY (activity_id)
		REFERENCES activity(id)
);