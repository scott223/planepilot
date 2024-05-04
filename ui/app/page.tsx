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
  const [viewports, setViewports] = useState(4); //default 4 viewports

  const handletfChange = (e: ChangeEvent<HTMLSelectElement>) => {
    setTimeframeMinutes(e.target.value as unknown as number); //typecast to number, 
  };

  const handlevpChange = (e: ChangeEvent<HTMLSelectElement>) => {
    setViewports(e.target.value as unknown as number); //typecast to number, 
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
          Timeframe (minutes): &nbsp;
          <select
            value={timeframeMinutes}
            onChange={handletfChange}
          >
            <option value="10">10</option>
            <option value="5">5</option>
            <option value="1">1</option>
          </select>
        </div>
        <div className="content-center">
          Viewports (number): &nbsp;
          <select
            value={viewports}
            onChange={handlevpChange}
          >
            <option value="8">8</option>
            <option value="4">4</option>
            <option value="2">2</option>
          </select>
        </div>
      </div>
      <div className="grid grid-cols-2 gap-4 mt-6">
        {Array.from({ length: viewports }, (e, i) => (
          <div key={i}><ChannelChart timeframeMinutes={timeframeMinutes} /></div>
        ))}
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
