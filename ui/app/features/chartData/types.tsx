export type chartDataType = {
    value: number,
    timestamp: Date,
    channel: number,
}

export type channelDataType = {
    id: number,
    title: string,
}

export interface chartStateType {
    chartData: chartDataType[],
    channels: channelDataType[],
    errors: string,
    isloading: boolean,
}

export enum chartDataActionTypes {
    FETCH_DATA_REQUEST = '@chartdata/FETCH_DATA_REQUEST',
    FETCH_DATA_SUCCESS = '@chartdata/FETCH_DATA_SUCCESS',
    FETCH_DATA_ERROR = '@chartdata/FETCH_DATA_ERROR',
}