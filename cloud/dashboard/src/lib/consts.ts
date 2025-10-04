export const LOGIN_ROUTES = ["/login", "/password", "/forgot-password", "/passkey"];

export const LANDING_ROUTE = "/projects";

export const OAUTH_CALLBACK_ROUTES = [
    "/login/magic-link",
    "/oauth2-callback/google",
    "/register/confirm",
] as const;
