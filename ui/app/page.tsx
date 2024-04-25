"use client"; // This is a client component 👈🏽
import React, { useRef, useState, useEffect, PureComponent } from 'react'
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';
import moment from 'moment';

import store from './store'
import { Provider } from 'react-redux'

import { ChannelChart } from './features/chartData/ChannelChart'

/*
export const ChannelViewer: React.FC<DataProps> = props => {
  const { data } = props;

  return (
    <ResponsiveContainer width="100%" height="100%">
      <LineChart
        width={500}
        height={300}
        data={data}
        margin={{
          top: 5,
          right: 30,
          left: 20,
          bottom: 5,
        }}
      >
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis
          domain={['auto', 'auto']}
          name="Time"
          tickFormatter={time => moment(time).format('MM/DD HH:mm')}
          reversed={true}
          dataKey="timestamp"
        />
        <YAxis />
        <Tooltip labelFormatter={time => moment(time).format('DD/MM HH:mm:SS')} />
        <Legend />
        <Line type="monotone" dataKey="value" stroke="#8884d8" activeDot={{ r: 8 }} />
      </LineChart>
    </ResponsiveContainer>
  );

}

*/

export default function Home() {

  return (
    <main className="flex min-h-screen flex-col items-center justify-between">
      <div className="border-solid border-2 border-indigo-600 w-full h-screen">
        <Provider store={store}>
          <ChannelChart />
          <ChannelChart />
          <ChannelChart />
          <ChannelChart />
        </Provider>
      </div>
    </main>
  );
}
