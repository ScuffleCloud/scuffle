CREATE TABLE "policy_sets" (
    "id" UUID PRIMARY KEY,
    -- Ref: organizations.id
    -- The organization can be null in that case the policy set is global for all organizations
    "organization_id" UUID,
    "data" BYTEA NOT NULL
);

ALTER TABLE "policy_sets"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id")
ON DELETE CASCADE;

CREATE INDEX ON "policy_sets"("organization_id");

CREATE TABLE "roles" (
    "id" UUID PRIMARY KEY,
    -- Ref: organizations.id
    "organization_id" UUID NOT NULL,
    "name" VARCHAR(255) NOT NULL,
    "description" TEXT,
    "inline_policy_set" BYTEA NOT NULL
);

ALTER TABLE "roles"
ADD FOREIGN KEY("organization_id") REFERENCES "organizations"("id")
ON DELETE CASCADE;

CREATE INDEX ON "roles"("organization_id");

CREATE TABLE "role_member_assignments" (
    -- Ref: organizations.id
    "organization_id" UUID NOT NULL,
    -- Ref: users.id
    "user_id" UUID NOT NULL,
    -- Ref: roles.id
    "role_id" UUID NOT NULL,
    PRIMARY KEY("organization_id", "user_id", "role_id")
);

ALTER TABLE "role_member_assignments"
ADD FOREIGN KEY("organization_id", "user_id") REFERENCES "organization_members"("organization_id", "user_id")
ON DELETE CASCADE;

CREATE INDEX ON "role_member_assignments"("role_id");

CREATE TABLE "role_policy_set_assignments" (
    -- Ref: roles.id
    "role_id" UUID NOT NULL,
    -- Ref: policy_sets.id
    "policy_set_id" UUID NOT NULL,
    PRIMARY KEY("role_id", "policy_set_id")
);

ALTER TABLE "role_policy_set_assignments"
ADD FOREIGN KEY("role_id") REFERENCES "roles"("id")
ON DELETE CASCADE;

ALTER TABLE "role_policy_set_assignments"
ADD FOREIGN KEY("policy_set_id") REFERENCES "policy_sets"("id")
ON DELETE CASCADE;

CREATE INDEX ON "role_policy_set_assignments"("policy_set_id");

CREATE TABLE "organization_member_policy_set_assignments" (
    -- Ref: organizations.id
    "organization_id" UUID NOT NULL,
    -- Ref: users.id
    "user_id" UUID NOT NULL,
    -- Ref: policy_sets.id
    "policy_set_id" UUID NOT NULL,
    PRIMARY KEY("organization_id", "user_id", "policy_set_id")
);

ALTER TABLE "organization_member_policy_set_assignments"
ADD FOREIGN KEY("organization_id", "user_id") REFERENCES "organization_members"("organization_id", "user_id")
ON DELETE CASCADE;

ALTER TABLE "organization_member_policy_set_assignments"
ADD FOREIGN KEY("policy_set_id") REFERENCES "policy_sets"("id")
ON DELETE CASCADE;

CREATE INDEX ON "organization_member_policy_set_assignments"("policy_set_id");

ALTER TABLE "organization_members"
ADD COLUMN "inline_policy" BYTEA;
