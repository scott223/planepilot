import { channelDataType, chartDataActionTypes, chartDataType } from './types'
import { action } from 'typesafe-actions';

export const fetchDataRequest = (channel: number) => action(chartDataActionTypes.FETCH_DATA_REQUEST, channel);
export const fetchDataSuccess = (data: chartDataType[]) => action(chartDataActionTypes.FETCH_DATA_SUCCESS, data);
export const fetchDataError = (message: string) => action(chartDataActionTypes.FETCH_DATA_ERROR, message);

export const fetchChannelRequest = () => action(chartDataActionTypes.FETCH_CHANNEL_REQUEST);
export const fetchChannelSuccess = (data: channelDataType[]) => action(chartDataActionTypes.FETCH_CHANNEL_SUCCESS, data);
export const fetchChannelError = (message: string) => action(chartDataActionTypes.FETCH_CHANNEL_ERROR, message);