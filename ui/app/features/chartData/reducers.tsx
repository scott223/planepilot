import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import { chartDataActionTypes, chartDataType, chartStateType } from './types'

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
        case chartDataActionTypes.FETCH_DATA_REQUEST: {
            console.log("Fetching data from API");
            return { ...state, isloading: true };
        }
        case chartDataActionTypes.FETCH_DATA_SUCCESS: {
            console.log(action);
            return { ...state, isloading: false, chartData: action.payload };
        }
        case chartDataActionTypes.FETCH_DATA_ERROR: {
            return { ...state, isloading: false, error: action.error };
        }
        default:
            return state
    }
}

//export const {
//    getDataAction,
//    getDataSuccessAction,
//    getDataErrorAction
//} = chartDataSlice.actions;

//export default chartDataSlice.reducer