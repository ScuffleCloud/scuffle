export const ssr = false;

import type { VideoStream } from "$lib/components/streams/types";
import type { ListResponse } from "$lib/types";
import { mockStreamsListResponse } from "../../../../../../../mocks/streams";

import type { PageLoad } from "./$types";

export const load = (async ({ depends }) => {
    depends("streams:data");

    const fetchStreams = async (): Promise<ListResponse<VideoStream>> => {
        return mockStreamsListResponse;
    };

    return {
        streams: fetchStreams(),
    };
}) satisfies PageLoad;
