CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE IF NOT EXISTS users (
	id SERIAL PRIMARY KEY,
	user_uuid UUID NOT NULL UNIQUE DEFAULT uuid_generate_v4(),
	username VARCHAR(64) NOT NULL,
	created_at TIMESTAMP NOT NULL DEFAULT NOW(),
	updated_at TIMESTAMP NOT NULL DEFAULT NOW(),
	UNIQUE(username)
);

CREATE TABLE IF NOT EXISTS pocket_articles (
	id SERIAL PRIMARY KEY,
	user_id INT REFERENCES users(id),
	item_id VARCHAR(20) NOT NULL,
	resolved_id VARCHAR(20),
	given_url VARCHAR(255),
	given_title VARCHAR(255),
	favorite BOOLEAN NOT NULL DEFAULT FALSE,
	status INT,
	time_added BIGINT,
	time_updated BIGINT,
	time_read BIGINT,
	time_favorited BIGINT,
	sort_id INT,
	resolved_url VARCHAR(255),
	resolved_title VARCHAR(255),
	excerpt TEXT,
	is_article BOOLEAN NOT NULL DEFAULT TRUE,
	is_index BOOLEAN NOT NULL DEFAULT FALSE,
	has_image INT,
	has_video INT,
	word_count INT,
	tags VARCHAR(255),
	lang VARCHAR(10),
	time_to_read INT,
	listen_duration_estimate INT,
	top_image_url VARCHAR(255),
	UNIQUE (user_id, item_id)
);

CREATE TABLE IF NOT EXISTS pocket_article_videos (
	id SERIAL PRIMARY KEY,
	pocket_article_id INT REFERENCES pocket_articles(id),
	item_id VARCHAR(20) NOT NULL,
	video_id VARCHAR(20) NOT NULL,
	src VARCHAR(255) NOT NULL,
	width INT NOT NULL,
	height INT NOT NULL,
	length INT,
	vid VARCHAR(255) NOT NULL,
	UNIQUE (pocket_article_id, item_id, video_id)
);

CREATE TABLE IF NOT EXISTS pocket_article_images (
	id SERIAL PRIMARY KEY,
	pocket_article_id INT REFERENCES pocket_articles(id),
	item_id VARCHAR(20) NOT NULL,
	image_id VARCHAR(20) NOT NULL,
	src VARCHAR(255) NOT NULL,
	width INT NOT NULL,
	height INT NOT NULL,
	caption TEXT NOT NULL DEFAULT '',
	credit TEXT NOT NULL DEFAULT '',
	UNIQUE (pocket_article_id, item_id, image_id)
);

CREATE TABLE IF NOT EXISTS pocket_article_authors (
	id SERIAL PRIMARY KEY,
	pocket_article_id INT REFERENCES pocket_articles(id),
	author_id VARCHAR(20) NOT NULL,
	name VARCHAR(255) NOT NULL,
	url VARCHAR(255) NOT NULL,
	UNIQUE (pocket_article_id, author_id)
);