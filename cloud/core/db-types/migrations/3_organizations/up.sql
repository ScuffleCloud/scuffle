-- Organization is a logical grouping
CREATE TABLE "organizations" (
    "id" UUID PRIMARY KEY,
    "name" VARCHAR(255) NOT NULL,
    "owner_id" UUID NOT NULL
);

ALTER TABLE "organizations"
ADD FOREIGN KEY("owner_id") REFERENCES "users"("id");

CREATE INDEX ON "organizations"("owner_id");

-- Project is a sub-division of an organization.
CREATE TABLE "projects" (
    "id" UUID PRIMARY KEY,
    "name" VARCHAR(255) NOT NULL,
    "organization_id" UUID NOT NULL
);

ALTER TABLE "projects"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id");

-- Organizations have users that can view and edit the organization.
CREATE TABLE "organization_members" (
    -- Ref: organizations.id
    "organization_id" UUID NOT NULL,
    -- Ref: users.id
    "user_id" UUID NOT NULL,
    -- Ref: users.id
    "invited_by_id" UUID,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY("organization_id", "user_id")
);

-- No need to create an index, since this is indexed by the primary key.
ALTER TABLE "organization_members"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id");

-- Reverse the primary key index.
CREATE INDEX ON "organization_members"("user_id", "organization_id");
ALTER TABLE "organization_members"
ADD FOREIGN KEY("user_id") REFERENCES "users"("id");

CREATE INDEX ON "organization_members"("invited_by_id");
ALTER TABLE "organization_members"
ADD FOREIGN KEY("invited_by_id") REFERENCES "users"("id");

CREATE TABLE "organization_invitations" (
    "id" UUID PRIMARY KEY,
    "organization_id" UUID NOT NULL,
    "email" VARCHAR(255) NOT NULL,
    "invited_by_id" UUID NOT NULL,
    "expires_at" TIMESTAMPTZ
);

ALTER TABLE "organization_invitations"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id")
ON DELETE CASCADE;

ALTER TABLE "organization_invitations"
ADD FOREIGN KEY("invited_by_id") REFERENCES "users"("id");

