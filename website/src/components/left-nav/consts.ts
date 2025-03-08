export const NAV_ITEMS = [
    {
        id: 'home',
        label: 'Home',
        path: '/',
        children: [],
    },
    {
        id: 'dashboard',
        label: 'Dashboard',
        path: '/dashboard',
        children: [
            { id: 'analytics', label: 'Analytics', path: '/dashboard/analytics' },
            { id: 'reports', label: 'Reports', path: '/dashboard/reports' },
            { id: 'metrics', label: 'Metrics', path: '/dashboard/metrics' },
        ],
    },
    {
        id: 'projects',
        label: 'Projects',
        path: '/projects',
        children: [
            { id: 'active', label: 'Active Projects', path: '/projects/active' },
            { id: 'archived', label: 'Archived', path: '/projects/archived' },
            { id: 'templates', label: 'Templates', path: '/projects/templates' },
        ],
    },
];
