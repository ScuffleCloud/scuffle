<!-- SO. Eventually we need to make a routing function that performs as follows
 - This page not be automatically routed to this page
 - If the user is currently authed but not 2fa'd, this page is a valid page
 - User can click back to go back to the base login page while still being 1/2'd authed
 - User accessing this page while fully authed should route to users home page
 - Maintain state based shallow routing because login flows with flickering pages looks bad
-->
This page shoudln't get hit. Keep here for comments. Just implement inside
+layout.svelte
