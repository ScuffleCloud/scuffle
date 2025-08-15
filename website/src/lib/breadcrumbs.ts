export type BreadcrumbType = 'link' | 'switcher' | 'text';

export type BreadcrumbConfig = {
    show: boolean;
    type: BreadcrumbType;
    title?: string;
};

export const breadcrumbPaths: Record<string, BreadcrumbConfig> = {
    '/organization/[orgId]': {
        show: true,
        type: 'switcher',
        title: 'Organization',
    },
    '/organization/[orgId]/project/[projectId]': {
        show: true,
        type: 'switcher',
        title: 'Project',
    },
    '/streams': {
        show: true,
        type: 'link',
        title: 'Streams',
    },
    '/streams/[id]': {
        show: true,
        type: 'text',
        title: 'Stream Details',
    },
};

export function shouldShowBreadcrumbs(pathname: string): boolean {
    const pathParts = pathname.split('/').map((part) => {
        return /^[0-9a-f-]+$/.test(part) ? '[id]' : part;
    });

    const fullPath = pathParts.join('/');
    return !!breadcrumbPaths[fullPath]?.show;
}

export function getBreadcrumbConfig(path: string): BreadcrumbConfig | undefined {
    return breadcrumbPaths[path];
}
