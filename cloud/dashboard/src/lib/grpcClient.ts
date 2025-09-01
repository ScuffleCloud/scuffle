import { PUBLIC_GRPC_BASE_URL } from "$env/static/public";
import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { OrganizationInvitationsServiceClient } from "@scufflecloud/proto/scufflecloud/core/v1/organization_invitations_service.client.js";
import { OrganizationsServiceClient } from "@scufflecloud/proto/scufflecloud/core/v1/organizations_service.client.js";
import { SessionsServiceClient } from "@scufflecloud/proto/scufflecloud/core/v1/sessions_service.client.js";
import { UsersServiceClient } from "@scufflecloud/proto/scufflecloud/core/v1/users_service.client.js";

const transport = new GrpcWebFetchTransport({
    baseUrl: PUBLIC_GRPC_BASE_URL,
    format: "binary",
});

export const sessionsServiceClient = new SessionsServiceClient(transport);
export const organizationsServiceClient = new OrganizationsServiceClient(transport);
export const usersServiceClient = new UsersServiceClient(transport);
export const organizationInvitationsServiceClient = new OrganizationInvitationsServiceClient(transport);
