import { configureStore } from '@reduxjs/toolkit'
import rootReducers from './root-reducers'
import rootSaga from "./root-sagas";

import createSagaMiddleware from 'redux-saga'

const sagaMiddleware = createSagaMiddleware();


const store = configureStore({
    // @ts-ignore
    reducer: rootReducers,
    // @ts-ignore
    middleware: (getDefaultMiddleware) => getDefaultMiddleware({ serializableCheck: false }).concat(sagaMiddleware),
});

sagaMiddleware.run(rootSaga);

export default store;

// Infer the `RootState` and `AppDispatch` types from the store itself
export type RootState = ReturnType<typeof store.getState>
// Inferred type: {posts: PostsState, comments: CommentsState, users: UsersState}
export type AppDispatch = typeof store.dispatch