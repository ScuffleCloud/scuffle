forbid(
    principal is User,
    action,
    resource
) when {
    context.user_session_mfa_pending == true
};

permit(
    principal is User,
    action == Action::"register_with_email_password",
    resource == Application::"core"
);

permit(
    principal is User,
    action == Action::"register_with_google",
    resource == Application::"core"
);

permit(
    principal is User,
    action == Action::"login_with_email_password",
    resource == Application::"core"
);

permit(
    principal is User,
    action == Action::"request_magic_link",
    resource == Application::"core"
);

permit(
    principal is User,
    action == Action::"login_with_magic_link",
    resource == Application::"core"
);

permit(
    principal is User,
    action == Action::"login_with_google",
    resource == Application::"core"
);

permit(
    principal is User,
    action == Action::"login_with_webauthn",
    resource == Application::"core"
);

permit(
    principal is User,
    action,
    resource is User
) when {
    principal.id == resource.id
};

permit(
    principal is User,
    action in [Action::"create_user_email", Action::"delete_user_email"],
    resource is UserEmail
) when {
    principal.id == resource.user_id
};

permit(
    principal is User,
    action == Action::"approve_user_session_request",
    resource is UserSessionRequest
);

permit(
    principal is User,
    action == Action::"refresh_user_session",
    resource is UserSession
) when {
    principal.id == resource.user_id
};

permit(
    principal is User,
    action == Action::"invalidate_user_session",
    resource is UserSession
) when {
    principal.id == resource.user_id
};
