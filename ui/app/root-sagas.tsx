import { all, fork } from "redux-saga/effects";
import { watchGetData } from "./features/chartData/sagas";

const rootSaga = function* () {
    yield all([
        fork(watchGetData),
        // Other forks
    ]);
};

export default rootSaga;