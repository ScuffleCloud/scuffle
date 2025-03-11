import type { NavItem } from './types';

export const NAV_ITEMS: NavItem[] = [
    // {
    //     id: 'home',
    //     label: 'Home',
    //     path: '/',
    //     children: [],
    // },
    {
        id: 'video',
        label: 'Video',
        path: '/video',
        children: [
            { id: 'streams', label: 'Streams', path: '/video/streams' },
            { id: 'reports', label: 'Reports', path: '/video/reports' },
            { id: 'metrics', label: 'Metrics', path: '/video/metrics' },
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
