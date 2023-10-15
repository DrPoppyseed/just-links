ALTER TABLE pocket_articles ALTER COLUMN given_url TYPE TEXT;
ALTER TABLE pocket_articles ALTER COLUMN given_title TYPE TEXT;
ALTER TABLE pocket_articles ALTER COLUMN resolved_url TYPE TEXT;
ALTER TABLE pocket_articles ALTER COLUMN resolved_title TYPE TEXT;
ALTER TABLE pocket_articles ALTER COLUMN tags TYPE TEXT;
ALTER TABLE pocket_articles ALTER COLUMN top_image_url TYPE TEXT;

ALTER TABLE pocket_article_videos ALTER COLUMN src TYPE TEXT;
ALTER TABLE pocket_article_videos ALTER COLUMN vid TYPE TEXT;

ALTER TABLE pocket_article_images ALTER COLUMN src TYPE TEXT;

ALTER TABLE pocket_article_authors ALTER COLUMN name TYPE TEXT;
ALTER TABLE pocket_article_authors ALTER COLUMN url TYPE TEXT;