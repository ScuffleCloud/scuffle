-- User is a person that can login and manage their account.
CREATE TABLE "users" (
    "id" UUID PRIMARY KEY,
    "preferred_name" VARCHAR(255),
    "first_name" VARCHAR(255),
    "last_name" VARCHAR(255),
    "password_hash" VARCHAR(255), -- Nullable for users who register via external providers
    "primary_email" VARCHAR(255) NOT NULL,
    "avatar_url" VARCHAR(255)
);

-- User emails are emails that are associated with a user.
CREATE TABLE "user_emails" (
    "email" VARCHAR(255) PRIMARY KEY,
    "user_id" UUID NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Pending emails are emails that are pending verification.
-- They are stored here so that the user can resend the code if they need to.
CREATE TABLE "pending_user_emails" (
    "email" VARCHAR(255) NOT NULL,
    "user_id" UUID NOT NULL,
    "codes_sent" INTEGER NOT NULL DEFAULT 0,
    "last_sent_code_at" TIMESTAMPTZ NOT NULL,
    PRIMARY KEY("email", "user_id")
);

CREATE INDEX ON "pending_user_emails"("user_id");

-- User connections to Google.
--
-- https://developers.google.com/people/api/rest/v1/people/get
-- https://developers.google.com/people/api/rest/v1/people#Person
-- Person.memberships.metadata.source
-- Also: Person.metadata.sources.profileMetadata.userTypes == GOOGLE_APPS_USER
CREATE TABLE "user_google_connections" (
    "sub" VARCHAR(255) PRIMARY KEY,
    "user_id" UUID NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX ON "user_google_connections"("user_id");

ALTER TABLE "user_google_connections"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id")
ON DELETE CASCADE;

