import type { NavItem } from './types';
import IconTest from '$lib/images/IconTest.svelte';
import IconTest2 from '$lib/images/IconTest2.svelte';
import IconStats from '$lib/images/IconStats.svelte';
export const NAV_ITEMS: NavItem[] = [
    {
        id: 'overview',
        label: 'Overview',
        path: '/overview',
        icon: IconStats,
    },
    {
        id: 'video',
        label: 'Video',
        path: '/video',
        icon: IconTest,
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
        icon: IconTest2,
        children: [
            { id: 'active', label: 'Active Projects', path: '/projects/active' },
            { id: 'archived', label: 'Archived', path: '/projects/archived' },
            { id: 'templates', label: 'Templates', path: '/projects/templates' },
        ],
    },
];
