/**
 * Get the parent route by removing the last segment from a pathname
 * @param pathname - The current pathname (e.g., '/streams/123/overview')
 * @param tabSegments - Optional array of known tab/child segments to remove
 * @returns The parent route (e.g., '/streams/123')
 */
export function getParentRoute(pathname: string, tabSegments?: string[]): string {
  const segments = pathname.split("/").filter(Boolean);

  if (segments.length === 0) return "/";

  // If tab segments are provided, only remove if last segment is a known tab
  if (tabSegments && tabSegments.length > 0) {
    const lastSegment = segments[segments.length - 1];
    if (tabSegments.includes(lastSegment)) {
      const parentSegments = segments.slice(0, -1);
      return parentSegments.length > 0 ? "/" + parentSegments.join("/") : "/";
    }
    return pathname;
  }

  // Default: just remove the last segment
  const parentSegments = segments.slice(0, -1);
  return parentSegments.length > 0 ? "/" + parentSegments.join("/") : "/";
}

/**
 * Get the current tab/segment from a pathname
 * @param pathname - The current pathname
 * @param validTabs - Array of valid tab identifiers
 * @param fallback - Fallback tab if current segment isn't valid (default: first tab)
 * @returns The current tab identifier
 */
export function getCurrentTab(pathname: string, validTabs: string[], fallback?: string): string {
  const segments = pathname.split("/").filter(Boolean);
  const lastSegment = segments[segments.length - 1];

  if (validTabs.includes(lastSegment)) {
    return lastSegment;
  }

  return fallback || validTabs[0] || "";
}

/**
 * Build a tab URL by combining parent route with tab segment
 * @param parentRoute - The parent route
 * @param tabId - The tab identifier
 * @returns The complete tab URL
 */
export function buildTabUrl(parentRoute: string, tabId: string): string {
  const cleanParent = parentRoute.endsWith("/") ? parentRoute.slice(0, -1) : parentRoute;
  return `${cleanParent}/${tabId}`;
}
