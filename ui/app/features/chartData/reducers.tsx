import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import { channelDataType, chartDataActionTypes, chartStateType } from './types'

const initialState: chartStateType = {
    chartData: [],
    channels: [],
    errors: '',
    isloading: false,
}

export default function chartDataReducer(state = initialState, action: {
    payload: any;
    error: string;
    type: chartDataActionTypes
}) {
    switch (action.type) {
        //fetch data
        case chartDataActionTypes.FETCH_DATA_REQUEST: {
            console.log("Fetching data from API");
            return { ...state, isloading: true };
        }
        case chartDataActionTypes.FETCH_DATA_SUCCESS: {
            console.log("Succesfully fetched data from API");
            return { ...state, isloading: false, chartData: action.payload };
        }
        case chartDataActionTypes.FETCH_DATA_ERROR: {
            return { ...state, isloading: false, error: action.error };
        }

        //fetch channels
        case chartDataActionTypes.FETCH_CHANNEL_REQUEST: {
            console.log("Fetching channels from API");
            return { ...state, isloading: true };
        }
        case chartDataActionTypes.FETCH_CHANNEL_SUCCESS: {
            console.log("Succesfully fetched channels from API");

            return { ...state, isloading: false, channels: action.payload };
        }
        case chartDataActionTypes.FETCH_CHANNEL_ERROR: {
            return { ...state, isloading: false, error: action.error };
        }
        default:
            return state
    }
}