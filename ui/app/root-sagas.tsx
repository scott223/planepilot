import { all, fork } from "redux-saga/effects";
import { watchGetData, watchGetChannel } from "./features/chartData/sagas";

const rootSaga = function* () {
    yield all([
        fork(watchGetData),
        fork(watchGetChannel),
        // Other forks
    ]);
};

export default rootSaga;