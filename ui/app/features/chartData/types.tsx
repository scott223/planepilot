export type chartDataType = {
    value: number,
    timestamp: Date,
    channel: number,
}

export type channelDataType = {
    channel_id: number,
    channel_name: string,
}

export interface chartStateType {
    chartData: chartDataType[],
    channels: channelDataType[],
    errors: string,
    isloading: boolean,
    lastSuccesfullLoad?: Date,
}

export enum chartDataActionTypes {
    FETCH_DATA_REQUEST = '@chartdata/FETCH_DATA_REQUEST',
    FETCH_DATA_SUCCESS = '@chartdata/FETCH_DATA_SUCCESS',
    FETCH_DATA_ERROR = '@chartdata/FETCH_DATA_ERROR',
    FETCH_CHANNEL_REQUEST = '@chartdata/FETCH_CHANNEL_REQUEST',
    FETCH_CHANNEL_SUCCESS = '@chartdata/FETCH_CHANNEL_SUCCESS',
    FETCH_CHANNEL_ERROR = '@chartdata/FETCH_CHANNEL_ERROR',
}