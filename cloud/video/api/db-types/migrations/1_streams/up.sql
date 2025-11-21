CREATE TABLE "streams" (
    "id" UUID PRIMARY KEY,
    "project_id" UUID NOT NULL,
    "name" VARCHAR(255) NOT NULL
);
