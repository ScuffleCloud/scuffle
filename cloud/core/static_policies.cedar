// Allow unauthenticated users to register via email
permit(
    principal is Unauthenticated,
    action == Action::"create",
    resource is EmailRegistrationRequest
);
permit(
    principal is Unauthenticated,
    action == Action::"create",
    resource is User
);

// Allow existing users to register a new email address
permit(
    principal is User,
    action == Action::"create",
    resource is EmailRegistrationRequest
);
permit(
    principal is User,
    action == Action::"create",
    resource is UserEmail
);

// Allow login
permit(
    principal is User,
    action == Action::"create",
    resource is UserSession
);

// Allow logout
permit(
    principal is User,
    action == Action::"delete",
    resource is UserSession
);

// Forbid actions when MFA is pending
forbid(
    principal is User,
    action,
    resource
) when {
    context.user_session.user_id == principal.id &&
    context.user_session.mfa_pending == true
} unless {
    action == Action::"update" && resource is UserSession
};
