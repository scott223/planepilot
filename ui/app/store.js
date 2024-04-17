import { configureStore } from '@reduxjs/toolkit'
import rootReducers from './root-reducers'
import rootSaga from "./root-sagas";

import createSagaMiddleware from 'redux-saga'

const sagaMiddleware = createSagaMiddleware();

const store = configureStore({
    reducer: rootReducers,
    middleware: (getDefaultMiddleware) => getDefaultMiddleware().concat(sagaMiddleware),
});

sagaMiddleware.run(rootSaga);

export default store;