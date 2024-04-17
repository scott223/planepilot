import { chartDataActionTypes, chartDataType, chartStateType } from './types'
import { action } from 'typesafe-actions';

export const fetchDataRequest = (channel: number) => action(chartDataActionTypes.FETCH_DATA_REQUEST, channel);
export const fetchDataSuccess = (data: chartDataType[]) => action(chartDataActionTypes.FETCH_DATA_SUCCESS, data);
export const fetchDataError = (message: string) => action(chartDataActionTypes.FETCH_DATA_ERROR, message);