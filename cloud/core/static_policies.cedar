forbid (
  principal is User,
  action,
  resource
)
when { context.user_session_mfa_pending == true }
unless
{
  action == Action::"create_webauthn_challenge" ||
  action == Action::"validate_mfa_for_user_session" ||
  action == Action::"invalidate_user_session" ||
  action == Action::"login_with_email_password" ||
  action == Action::"request_magic_link" ||
  action == Action::"login_with_magic_link" ||
  action == Action::"login_with_google" ||
  action == Action::"login_with_webauthn"
};

permit (
  principal is User,
  action == Action::"login_with_email_password",
  resource == Application::"core"
);

permit (
  principal is Unauthenticated,
  action == Action::"request_magic_link",
  resource == Application::"core"
);

permit (
  principal is User,
  action == Action::"login_with_magic_link",
  resource == Application::"core"
);

permit (
  principal is User,
  action == Action::"login_with_google",
  resource == Application::"core"
);

permit (
  principal is User,
  action == Action::"login_with_webauthn",
  resource == Application::"core"
);

permit (
  principal is User,
  action == Action::"get_user",
  resource is User
)
when { principal.id == resource.id };

permit (
  principal is User,
  action == Action::"update_user",
  resource is User
)
when { principal.id == resource.id };

permit (
  principal is User,
  action == Action::"list_user_emails",
  resource is User
)
when { principal.id == resource.id };

permit (
  principal is User,
  action == Action::"create_user_email",
  resource is UserEmail
)
when { principal.id == resource.user_id };

permit (
  principal is User,
  action == Action::"delete_user_email",
  resource is UserEmail
)
when { principal.id == resource.user_id };

permit (
  principal is User,
  action == Action::"create_webauthn_credential",
  resource is User
)
when { principal.id == resource.id };

permit (
  principal is User,
  action == Action::"complete_create_webauthn_credential",
  resource is MfaWebauthnCredential
)
when { principal.id == resource.user_id };

permit (
  principal is User,
  action == Action::"update_webauthn_credential",
  resource is MfaWebauthnCredential
)
when { principal.id == resource.user_id };

permit (
  principal is User,
  action == Action::"create_webauthn_challenge",
  resource is User
)
when { principal.id == resource.id };

permit (
  principal is User,
  action == Action::"list_webauthn_credentials",
  resource is User
)
when { principal.id == resource.id };

permit (
  principal is User,
  action == Action::"delete_webauthn_credential",
  resource is MfaWebauthnCredential
)
when { principal.id == resource.user_id };

permit (
  principal is User,
  action == Action::"create_totp_credential",
  resource is User
)
when { principal.id == resource.id };

permit (
  principal is User,
  action == Action::"complete_create_totp_credential",
  resource is MfaTotpCredential
)
when { principal.id == resource.user_id };

permit (
  principal is User,
  action == Action::"update_totp_credential",
  resource is MfaTotpCredential
)
when { principal.id == resource.user_id };

permit (
  principal is User,
  action == Action::"delete_totp_credential",
  resource is MfaTotpCredential
)
when { principal.id == resource.user_id };

permit (
  principal is User,
  action == Action::"list_totp_credentials",
  resource is User
)
when { principal.id == resource.id };

permit (
  principal is User,
  action == Action::"regenerate_recovery_codes",
  resource is User
)
when { principal.id == resource.id };

permit (
  principal is User,
  action == Action::"delete_user",
  resource is User
)
when { principal.id == resource.id };

permit (
  principal is Unauthenticated,
  action == Action::"create_user_session_request",
  resource is UserSessionRequest
);

permit (
  principal is Unauthenticated,
  action == Action::"get_user_session_request",
  resource is UserSessionRequest
);

permit (
  principal is User,
  action == Action::"approve_user_session_request",
  resource is UserSessionRequest
);

permit (
  principal is Unauthenticated,
  action == Action::"complete_user_session_request",
  resource is UserSessionRequest
);

permit (
  principal is User,
  action == Action::"validate_mfa_for_user_session",
  resource is UserSession
)
when { principal.id == resource.user_id };

permit (
  principal is User,
  action == Action::"refresh_user_session",
  resource is UserSession
)
when { principal.id == resource.user_id };

permit (
  principal is User,
  action == Action::"invalidate_user_session",
  resource is UserSession
)
when { principal.id == resource.user_id };

permit (
  principal is User,
  action == Action::"create_organization",
  resource is Organization
)
when { principal.id == resource.owner_id };

permit (
  principal is User,
  action == Action::"get_organization",
  resource is Organization
)
when { principal in resource };

permit (
  principal is User,
  action == Action::"update_organization",
  resource is Organization
)
when { principal.id == resource.owner_id };

permit (
  principal is User,
  action == Action::"list_organization_members",
  resource is Organization
)
when { principal in resource };

permit (
  principal is User,
  action == Action::"list_organizations_by_user",
  resource is User
)
when { principal.id == resource.id };

permit (
  principal is User,
  action == Action::"create_project",
  resource is Project
)
when { principal in resource };

permit (
  principal is User,
  action == Action::"list_projects",
  resource is Organization
)
when { principal in resource };

permit (
  principal is User,
  action == Action::"create_organization_invitation",
  resource is OrganizationInvitation
)
when { principal.id == resource.organization.owner_id };

permit (
  principal is User,
  action == Action::"list_organization_invitations_by_user",
  resource is User
)
when { principal.id == resource.id };

permit (
  principal is User,
  action == Action::"list_organization_invitations_by_organization",
  resource is Organization
)
when { principal.id == resource.owner_id };

permit (
  principal is User,
  action == Action::"get_organization_invitation",
  resource is OrganizationInvitation
)
when
{
  (resource has user_id && principal.id == resource.user_id) ||
  principal.id == resource.organization.owner_id
};

permit (
  principal is User,
  action == Action::"accept_organization_invitation",
  resource is OrganizationInvitation
)
when { resource has user_id && principal.id == resource.user_id };

permit (
  principal is User,
  action == Action::"decline_organization_invitation",
  resource is OrganizationInvitation
)
when
{
  (resource has user_id && principal.id == resource.user_id) ||
  principal.id == resource.organization.owner_id
};
