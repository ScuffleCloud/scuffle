/**
 * Tracks whether the user has navigated within the app
 * with shallow routing or regular navigation
 */
export function useInternalNavigation() {
    let hasInternalHistory = $state(false);

    return {
        get hasInternalHistory() {
            return hasInternalHistory;
        },
        markInternalNavigation() {
            hasInternalHistory = true;
        },
    };
}

/**
 * Smart back navigation that falls back to a specified route
 * if no internal navigation history exists
 */
export function createSmartBack<T extends string>(fallbackRoute: T, navigateToRoute: (route: T) => void) {
    const navigation = useInternalNavigation();

    return {
        back: (specificFallback?: T) => {
            if (navigation.hasInternalHistory) {
                history.back();
            } else {
                const target = specificFallback || fallbackRoute;
                navigateToRoute(target);
            }
        },
        // Call to track internal navigation in shallow routing
        markNavigation: navigation.markInternalNavigation,
    };
}
