"use client"; // This is a client component
import React, { useRef, useState, useEffect, ChangeEvent } from 'react'
import { Provider, useDispatch } from 'react-redux'
import store from './store'

import { ChannelChart } from './features/chartData/ChannelChart'
import { fetchChannelRequest, fetchDataRequest } from './features/chartData/actions';

interface DataProps {
  //no props
}

const PlaneDashboard: React.FC<DataProps> = props => {

  const dispatch = useDispatch();

  dispatch(fetchChannelRequest());

  useEffect(() => {
    const interval = setInterval(() => {
      dispatch(fetchDataRequest())
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  const [timeframeMinutes, setTimeframeMinutes] = useState(10); //default 10 min

  const handleChange = (e: ChangeEvent<HTMLSelectElement>) => {
    setTimeframeMinutes(e.target.value as unknown as number); //typecast to number, 
  };

  return (
    <div>
      <div className="flex space-x-4 ...">
        <div>
          <button className="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded" type="button" onClick={() => dispatch(fetchChannelRequest())}>
            Update channels
          </button>
        </div>
        <div className="content-center">
          Timeframe (minutes):
          <select
            value={timeframeMinutes}
            onChange={handleChange}
          >
            <option value="10">10</option>
            <option value="5">5</option>
            <option value="1">1</option>
          </select>
        </div>
      </div>
      <div className="grid grid-cols-2 gap-4 mt-6">
        <div><ChannelChart timeframeMinutes={timeframeMinutes} /></div>
        <div><ChannelChart timeframeMinutes={timeframeMinutes} /></div>
        <div><ChannelChart timeframeMinutes={timeframeMinutes} /></div>
        <div><ChannelChart timeframeMinutes={timeframeMinutes} /></div>
      </div>
    </div>
  );
}

export default function Home() {

  return (
    <main className="flex min-h-screen flex-col items-center justify-between">
      <div className="border-solid w-5/6 h-screen mt-6">
        <Provider store={store}>
          <PlaneDashboard />
        </Provider>
      </div>
    </main >
  );
}
