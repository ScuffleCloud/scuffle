import { browser } from "$app/environment";
import { PUBLIC_GRPC_BASE_URL } from "$env/static/public";
import { GrpcWebFetchTransport } from "@protobuf-ts/grpcweb-transport";
import { base64decode } from "@protobuf-ts/runtime";
import {
    Deferred,
    type RpcError,
    type RpcInterceptor,
    type RpcMetadata,
    type RpcOptions,
    type RpcStatus,
    UnaryCall,
} from "@protobuf-ts/runtime-rpc";
import { DebugInfo } from "@scufflecloud/proto/google/rpc/error_details.js";
import { Status } from "@scufflecloud/proto/google/rpc/status.js";
import { OrganizationInvitationsServiceClient } from "@scufflecloud/proto/scufflecloud/core/v1/organization_invitations_service.client.js";
import { OrganizationsServiceClient } from "@scufflecloud/proto/scufflecloud/core/v1/organizations_service.client.js";
import { SessionsServiceClient } from "@scufflecloud/proto/scufflecloud/core/v1/sessions_service.client.js";
import { UsersServiceClient } from "@scufflecloud/proto/scufflecloud/core/v1/users_service.client.js";
import { authState } from "./auth.svelte";
import { arrayBufferToBase64 } from "./utils";

function generateRandomNonce(): ArrayBuffer {
    if (!browser) throw new Error("Not in browser");

    const array = new Uint8Array(32);
    window.crypto.getRandomValues(array);
    return array.buffer;
}

// https://github.com/timostamm/protobuf-ts/issues/628#issuecomment-1999999960
export function rpcErrorToString(error: RpcError): string {
    let status = null;
    const statusMeta = error.meta["grpc-status-details-bin"];
    if (statusMeta) {
        const b64StatusBin = typeof statusMeta === "string" ? statusMeta : statusMeta[statusMeta.length - 1];
        status = Status.fromBinary(base64decode(b64StatusBin));
    }

    if (status) {
        let s = `code ${status.code}: ${status.message}`;
        const debugInfo = status.details.find(d => d.typeUrl === "type.googleapis.com/google.rpc.DebugInfo")?.value;
        if (debugInfo) {
            const debugDetail = DebugInfo.fromBinary(debugInfo).detail;
            s += `\nDebug info: ${debugDetail}`;
        }
        return s;
    } else {
        return decodeURIComponent(error.message);
    }
}

const authInterceptor: RpcInterceptor = {
    interceptUnary(next, method, input, options: RpcOptions): UnaryCall {
        // https://github.com/timostamm/protobuf-ts/issues/580
        const defHeader = new Deferred<RpcMetadata>();
        const defMessage = new Deferred<object>();
        const defStatus = new Deferred<RpcStatus>();
        const defTrailer = new Deferred<RpcMetadata>();

        (async () => {
            if (!options.meta) {
                options.meta = {};
            }
            const auth = authState();
            if (!options.skipValidityCheck) {
                auth.checkValidity().catch((e) => {
                    defStatus.rejectPending(e);
                    defTrailer.rejectPending(e);
                    defHeader.rejectPending(e);
                    defMessage.rejectPending(e);
                });
            }

            if (auth.userSessionToken.state === "authenticated") {
                const tokenId = auth.userSessionToken.data.id;
                const timestamp = Date.now().toString();
                const nonce = arrayBufferToBase64(generateRandomNonce());

                const key = await window.crypto.subtle.importKey(
                    "raw",
                    auth.userSessionToken.data.token,
                    {
                        name: "HMAC",
                        hash: "SHA-256",
                    },
                    false,
                    ["sign"],
                );

                const source = tokenId + timestamp + nonce;
                const sourceBytes = new TextEncoder().encode(source);
                const hmac = await window.crypto.subtle.sign("HMAC", key, sourceBytes);

                options.meta["scuf-token-id"] = tokenId;
                options.meta["scuf-timestamp"] = timestamp;
                options.meta["scuf-nonce"] = nonce;
                options.meta["scuf-auth-method"] = "HMAC-SHA256;scuf-token-id,scuf-timestamp,scuf-nonce";
                options.meta["scuf-auth-hmac"] = arrayBufferToBase64(hmac);
            }

            try {
                const result = next(method, input, options);
                defHeader.resolve(await result.headers);
                defMessage.resolve(await result.response);
                defStatus.resolve(await result.status);
                defTrailer.resolve(await result.trailers);
            } catch (e) {
                defStatus.rejectPending(e);
                defTrailer.rejectPending(e);
                defHeader.rejectPending(e);
                defMessage.rejectPending(e);
            }
        })();

        return new UnaryCall(
            method,
            options.meta ?? {},
            input,
            defHeader.promise,
            defMessage.promise,
            defStatus.promise,
            defTrailer.promise,
        );
    },
};

const transport = new GrpcWebFetchTransport({
    baseUrl: PUBLIC_GRPC_BASE_URL,
    format: "binary",
    interceptors: [
        authInterceptor,
    ],
});

export const sessionsServiceClient = new SessionsServiceClient(transport);
export const organizationsServiceClient = new OrganizationsServiceClient(transport);
export const usersServiceClient = new UsersServiceClient(transport);
export const organizationInvitationsServiceClient = new OrganizationInvitationsServiceClient(transport);
