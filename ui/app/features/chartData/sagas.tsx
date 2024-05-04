import { PayloadAction } from "@reduxjs/toolkit";
import axios, { AxiosResponse } from "axios";
import { put, takeLatest } from "redux-saga/effects";
import { chartDataType, chartDataActionTypes, channelDataType } from "./types";
import { fetchDataSuccess, fetchDataError, fetchChannelSuccess, fetchChannelError } from "./actions";

// Generator function
function* getDataSaga({ payload: channel }: PayloadAction<number>) {
    try {
        // get the data
        const response: AxiosResponse<chartDataType[]> = yield axios.get(`http://localhost:3000/api/v1/data`);
        yield put(fetchDataSuccess(response.data));
    } catch (error) {
        yield put(fetchDataError(error as string));
    }
}

// Generator function
function* getChannelsSaga() {
    try {
        // get the channels
        const response: AxiosResponse<channelDataType[]> = yield axios.get(`http://localhost:3000/api/v1/channel`);
        yield put(fetchChannelSuccess(response.data));
    } catch (error) {
        yield put(fetchChannelError(error as string));
    }
}

// Generator function
export function* watchGetData() {
    //takelatest ignores all the calls, except the latest
    yield takeLatest(chartDataActionTypes.FETCH_DATA_REQUEST, getDataSaga);
}

// Generator function
export function* watchGetChannel() {
    yield takeLatest(chartDataActionTypes.FETCH_CHANNEL_REQUEST, getChannelsSaga);
}