export const load = async () => {
    // So if there's already a user session token. Redirect to dashboard

    // If user is authed but not 2fa'd, stay here

    // If someone hits /mfa directly but doesn't need MFA, redirect
    // if (
    //     auth.userSessionToken.state === "authenticated"
    //     && !auth.userSessionToken.data.
    // ) {
    //     throw redirect(302, "/dashboard");
    // }

    // if (auth.userSessionToken.state === "unauthenticated") {
    //     throw redirect(302, "/login");
    // }

    return {};
};
