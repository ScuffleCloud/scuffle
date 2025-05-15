import {
    graphic,
    type CustomSeriesRenderItemAPI,
    type CustomSeriesRenderItemParams,
} from 'echarts';

export const renderCircle = (
    params: CustomSeriesRenderItemParams,
    api: CustomSeriesRenderItemAPI,
) => {
    if (!api.size || !api.coord) return null;
    const categoryIndex = api.value(0);
    const value = api.coord([api.value(1), categoryIndex]);
    const size = (api.size([0, 1]) as number[])[1];
    const height = size * 0.65;

    const circleRadius = height / 2;

    const circleBox = {
        x: value[0] - circleRadius,
        y: value[1] - circleRadius,
        width: circleRadius * 2,
        height: circleRadius * 2,
    };

    const clippedShape = graphic.clipRectByRect(circleBox, {
        x: (params.coordSys as any).x,
        y: (params.coordSys as any).y,
        width: (params.coordSys as any).width,
        height: (params.coordSys as any).height,
    });

    if (!clippedShape) return null;

    return {
        type: 'circle',
        shape: {
            cx: value[0],
            cy: value[1],
            r: circleRadius,
        },
        style: {
            stroke: '#2b7fa4',
            lineWidth: 1,
            fill: '#83D472BF',
        },
        clipPath: {
            type: 'rect',
            shape: {
                x: clippedShape.x,
                y: clippedShape.y,
                width: clippedShape.width,
                height: clippedShape.height,
            },
        },
    };
};

export const renderDiamond = (
    params: CustomSeriesRenderItemParams,
    api: CustomSeriesRenderItemAPI,
) => {
    if (!api.size || !api.coord) return null;
    const categoryIndex = api.value(0);
    const value = api.coord([api.value(1), categoryIndex]);
    const size = (api.size([0, 1]) as number[])[1];
    const height = size * 0.75;

    // Calculate diamond points
    const diamondSize = height; // This will be the height/width before rotation

    const points = [
        [value[0], value[1] - diamondSize / 2], // top point
        [value[0] + diamondSize / 2, value[1]], // right point
        [value[0], value[1] + diamondSize / 2], // bottom point
        [value[0] - diamondSize / 2, value[1]], // left point
    ];

    // Bounding box for clipping
    const diamondBox = {
        x: value[0] - diamondSize / 2,
        y: value[1] - diamondSize / 2,
        width: diamondSize,
        height: diamondSize,
    };

    const clippedShape = graphic.clipRectByRect(diamondBox, {
        x: (params.coordSys as any).x,
        y: (params.coordSys as any).y,
        width: (params.coordSys as any).width,
        height: (params.coordSys as any).height,
    });

    if (!clippedShape) return null;

    return {
        type: 'polygon',
        shape: {
            points: points,
        },
        style: {
            ...api.style(),
            stroke: '#D44F34',
            lineWidth: 1,
            fill: '#FF9985BF',
            // border color
            borderWidth: 1,
        },
        clipPath: {
            type: 'rect',
            shape: {
                x: clippedShape.x,
                y: clippedShape.y,
                width: clippedShape.width,
                height: clippedShape.height,
            },
        },
    };
};

export const renderRectangle = (
    params: CustomSeriesRenderItemParams,
    api: CustomSeriesRenderItemAPI,
) => {
    if (!api.size || !api.coord) return null;
    const categoryIndex = api.value(0);
    const start = api.coord([api.value(1), categoryIndex]);
    const end = api.coord([api.value(2), categoryIndex]);

    const size = (api.size([0, 1]) as number[])[1];
    const height = size * 0.5;

    const rectShape = graphic.clipRectByRect(
        {
            x: start[0],
            y: start[1] - height / 2,
            width: end[0] - start[0],
            height: height,
        },
        {
            x: (params.coordSys as any).x,
            y: (params.coordSys as any).y,
            width: (params.coordSys as any).width,
            height: (params.coordSys as any).height,
        },
    );

    if (!rectShape) return null;

    return {
        type: 'rect',
        transition: ['shape'],
        shape: {
            ...rectShape,
            r: 2,
        },
        style: {
            ...api.style(),
            stroke: '#2b7fa4',
            lineWidth: 1,
        },
    };
};

export const renderItem = (
    params: CustomSeriesRenderItemParams,
    api: CustomSeriesRenderItemAPI,
) => {
    if (api.value(0) === 0) {
        return renderDiamond(params, api);
    } else if (api.value(0) === 1) {
        return renderCircle(params, api);
    }
    return renderRectangle(params, api);
};
