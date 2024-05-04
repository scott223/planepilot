import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import { channelDataType, chartDataActionTypes, chartDataType, chartStateType } from './types'
import { act } from 'react-dom/test-utils';

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
            //console.log("Fetching data from API");
            return { ...state, isloading: true };
        }
        case chartDataActionTypes.FETCH_DATA_SUCCESS: {
            //console.log("Succesfully fetched data from API");

            let arrayChartDataInLocalTimeZone: chartDataType[] = action.payload as chartDataType[]; //typecast into new array
            arrayChartDataInLocalTimeZone.forEach((element, index) => {
                arrayChartDataInLocalTimeZone[index].timestamp = new Date(element.timestamp); //this will automatically cast to local time
            });

            return { ...state, isloading: false, lastSuccesfullLoad: new Date(), chartData: arrayChartDataInLocalTimeZone };
        }
        case chartDataActionTypes.FETCH_DATA_ERROR: {
            return { ...state, isloading: false, error: action.error };
        }

        //fetch channels
        case chartDataActionTypes.FETCH_CHANNEL_REQUEST: {
            //console.log("Fetching channels from API");
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