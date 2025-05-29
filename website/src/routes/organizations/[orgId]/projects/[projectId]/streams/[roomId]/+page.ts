import { redirect } from '@sveltejs/kit';

export function load({ params }) {
    throw redirect(307, `${params.roomId}/overview`);
}
