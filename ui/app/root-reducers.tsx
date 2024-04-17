import counterReducer from './features/counter/counterSlice'
import chartDataReducer from './features/chartData/reducers'
import { chartStateType } from './features/chartData/types';

export type StateType = {
    chartDataState: chartStateType,
};

const rootReducers = {
    counter: counterReducer,
    chartData: chartDataReducer,
};

export default rootReducers;