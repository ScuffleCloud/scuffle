import { times } from 'lodash';

const interval = 1000;
const numRecords = 30;
const startTime = new Date('2024-01-01T00:00:00').getTime();

const RECTANGLE_INDEX = 2;
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
