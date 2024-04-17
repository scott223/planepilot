import { PayloadAction } from "@reduxjs/toolkit";
import axios, { AxiosResponse } from "axios";
import { put, takeLatest } from "redux-saga/effects";
import { chartDataType, chartDataActionTypes } from "./types";
import { fetchDataSuccess, fetchDataError } from "./actions";

// Generator function
function* getDataSaga({ payload: channel }: PayloadAction<number>) {
    try {
        // You can also export the axios call as a function.
        const response: AxiosResponse<chartDataType[]> = yield axios.get(`http://localhost:3000/api/v1/channel/${channel}/data`);
        yield put(fetchDataSuccess(response.data));
    } catch (error) {
        yield put(fetchDataError(error as string));
    }
}

// Generator function
export function* watchGetData() {
    yield takeLatest(chartDataActionTypes.FETCH_DATA_REQUEST, getDataSaga);
}