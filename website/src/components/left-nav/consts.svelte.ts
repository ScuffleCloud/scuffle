import type { NavItem } from '../types';
import IconTest from '$lib/images/IconTest.svelte';
import IconTest2 from '$lib/images/IconTest2.svelte';
import IconStats from '$lib/images/IconStats.svelte';
import Settings2 from '$lib/images/icon-settings2.svelte';

export const NAV_ITEMS: NavItem[] = [
    {
        id: 'overview',
        label: 'Overview',
        path: '/overview',
        icon: IconStats,
    },
    {
        id: 'streams',
        label: 'Streams',
        path: '/streams',
        icon: IconTest,
    },
    {
        id: 'assets',
        label: 'Assets',
        path: '/assets',
        icon: IconTest2,
    },
    {
        id: 'settings',
        label: 'Settings',
        path: '/settings',
        icon: Settings2,
        children: [
            { id: 'user', label: 'User', path: '/settings/user' },
            { id: 'organizations', label: 'Organizations', path: '/settings/organizations' },
        ],
    },
];
