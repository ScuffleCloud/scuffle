import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
    plugins: [sveltekit()],
    server: {
        allowedHosts: ['.scuf.dev'],
    },
    build: {
        rollupOptions: {
            external: (id) => {
                // Exclude MSW mocking library from prod builds
                if (id.includes('msw') && process.env.NODE_ENV === 'production') {
                    return true;
                }
                return false;
            },
        },
    },
    define: {
        global: 'globalThis',
    },
});
