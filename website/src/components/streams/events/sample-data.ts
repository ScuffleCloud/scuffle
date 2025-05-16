import { times } from 'lodash';

const interval = 1000;
const numRecords = 30;
const startTime = new Date('2024-01-01T00:00:00').getTime();

const RECTANGLE_INDEX = 2;
const TRIANGLE_INDEX = 1;
const CIRCLE_INDEX = 0;

// For rectangle
export const rectangleData = () => {
    const data = [];
    let baseTime = startTime;
    for (let i = 0; i < numRecords; i++) {
        let duration = Math.round(Math.random() * 10000);

        const value = [RECTANGLE_INDEX, baseTime, (baseTime += duration), duration];
        data.push({
            name: 'row1',
            value: value,
        });
        baseTime += Math.round(Math.random() * 2000);
    }
    return data;
};

export const triangleData = () => {
    const data = [];
    let baseTime = startTime;
    for (let i = 0; i < numRecords; i++) {
        let duration = Math.round(Math.random() * 10000);

        const value = [TRIANGLE_INDEX, (baseTime += duration)];
        data.push({
            name: 'row2',
            value: value,
        });
        baseTime += Math.round(Math.random() * 2000);
    }
    return data;
};

export const circleData = () => {
    const data = [];
    let baseTime = startTime;
    for (let i = 0; i < numRecords; i++) {
        let duration = Math.round(Math.random() * 10000);

        const value = [CIRCLE_INDEX, (baseTime += duration)];
        data.push({
            name: 'row3',
            value: value,
            // itemStyle: {
            //     normal: {
            //         color: '#000000',
            //     },
            // },
        });
        baseTime += Math.round(Math.random() * 2000);
    }
    return data;
};

// Let's build some line chart data but use signifiacntly more data points for a stream of data
const LINE_NUM_RECORDS = 500;

export const getLineData = () => {
    const data = [];
    let baseTime = startTime;
    for (let i = 0; i < LINE_NUM_RECORDS; i++) {
        const duration = Math.round(Math.random() * 1000);
        data.push({
            timestamp: baseTime,
            value: duration,
        });
        baseTime += Math.round(Math.random() * 2000);
    }
    return data;
};
